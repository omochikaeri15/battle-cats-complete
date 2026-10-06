use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use iced::futures::channel::mpsc;
use iced::widget::{column, container, markdown, pick_list, progress_bar, row, scrollable, text, text_input, Space};
use iced::{Alignment, Color, Element, Length, Size, Task, Theme};
use tracing::{error, info, warn};

use kore::domains::import;
use kore::domains::settings::source::{self, ErrorKind, Stamp, Step};
use kore::domains::settings::{FilesSettings, Settings as CoreSettings, UpdateMode};

use crate::app::state::SourceState;
use crate::app::theme;
use crate::common::feedback::{Slot, CONFIRM_SHORT_LABEL};
use crate::common::{digest, watcher};
use crate::widget::{hover_hint, popup, smooth_scroll};

use super::disk::format_size;

const ACKNOWLEDGEMENT: &str = r#"
The purpose of this agreement is to ensure that you, the User, are aware of the potential quirks regarding the use of a Provider.
By accepting this agreement, you must have read, understood, and accepted the following facts about Providers:

- Providers are incompatible with the "Mining" page, and will wipe its data.
- Providers wipe the import manifest, so future imports will import every file.
- Providers always overwrite every game file, so edits to vanilla will be lost.

You can accept the agreement by clicking the "Agree" button below. Selecting "Disagree" will close this pop-up.
"#;
const POPUP: popup::Spec = popup::Spec::new(popup::Kind::Source, Size::new(460.0, 270.0));
const TERMS_POPUP: popup::Spec = popup::Spec::new(popup::Kind::Provider, Size::new(560.0, 420.0));
const TERMS_PADDING: f32 = 20.0;
const SCROLLBAR_GAP: f32 = 8.0;
const CHOICE_SPACING: f32 = 12.0;
const CHOICE_GAP: f32 = 18.0;
const POPUP_PADDING: f32 = 20.0;
const CONTENT_SPACING: f32 = 14.0;
const BUTTON_SPACING: f32 = 12.0;
const PROGRESS_WIDTH: f32 = 300.0;
const PROGRESS_HEIGHT: f32 = 12.0;
const TITLE_SIZE: f32 = 18.0;
const BODY_SIZE: f32 = 14.0;
const SUBTLE_ALPHA: f32 = 0.6;
const INPUT_WIDTH: f32 = 440.0;
const MODES: [&str; 2] = ["Prompt", "Ignore"];
const LINK_HINT: &str = "A Provider link to an asset payload, served through a folder\nChecked on launch, and newer files are offered as a download";
const MODE_HINT: &str = "Prompt asks before downloading newer files found on launch\nIgnore never checks on launch";

#[derive(Debug, Clone)]
pub enum Message {
    LinkChanged(String),
    ModeSelected(UpdateMode),
    Check { manual: bool },
    Checked { manual: bool, link: String, result: Result<Stamp, source::Error> },
    Show,
    Accept,
    Decline,
    Ignore,
    IgnoreExpired,
    Cancel,
    Stepped(Step),
    Finished(Result<Stamp, source::Error>),
    Acknowledge,
    Popup(popup::Message),
    BannerExpired,
    Agree,
    Disagree,
    Terms(popup::Message),
    OpenUrl(String),
}

struct Progress {
    heading: &'static str,
    fraction: f32,
    detail: String,
    sealed: bool,
}

#[derive(Default)]
enum Status {
    #[default]
    Idle,
    Checking,
    Current,
    Found(String),
    Installing(Progress),
    Installed,
    Failed(ErrorKind),
}

pub struct State {
    is_open: bool,
    popup: popup::State,
    status: Status,
    abort: Arc<AtomicBool>,
    confirm: Slot<()>,
    banner: Slot<()>,
    refreshed: bool,
    terms_open: bool,
    terms_popup: popup::State,
    terms: Vec<markdown::Item>,
    pending: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            is_open: false,
            popup: popup::State::default(),
            status: Status::default(),
            abort: Arc::default(),
            confirm: Slot::default(),
            banner: Slot::default(),
            refreshed: false,
            terms_open: false,
            terms_popup: popup::State::default(),
            terms: crate::common::markdown::parse(ACKNOWLEDGEMENT),
            pending: None,
        }
    }
}

fn agreement() -> String {
    digest::hash(&[ACKNOWLEDGEMENT])
}

fn uploaded(modified: &str) -> &str {
    let dated = modified.split_once(", ").map_or(modified, |(_, rest)| rest);

    dated.match_indices(' ').nth(2).map_or(dated, |(end, _)| &dated[..end])
}

fn summary(stamp: &Stamp) -> String {
    match (stamp.modified.is_empty(), stamp.size) {
        (true, 0) => "Uploaded at an unknown time".to_owned(),
        (true, size) => format!("{} download", format_size(size)),
        (false, 0) => format!("Uploaded {}", uploaded(&stamp.modified)),
        (false, size) => format!("Uploaded {} ({})", uploaded(&stamp.modified), format_size(size)),
    }
}

fn progress(step: Step) -> Progress {
    match step {
        Step::Downloading { done, total: 0 } => Progress {
            heading: "Downloading game files",
            fraction: 0.0,
            detail: format_size(done),
            sealed: false,
        },
        Step::Downloading { done, total } => Progress {
            heading: "Downloading game files",
            fraction: done as f32 / total as f32,
            detail: format!("{} of {}", format_size(done), format_size(total)),
            sealed: false,
        },
        Step::Replacing => Progress {
            heading: "Replacing game files",
            fraction: 1.0,
            detail: "Swapping in the new \"game\" folder".to_owned(),
            sealed: true,
        },
    }
}

fn failure(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::Link => "That is not a Provider link this app can read",
        ErrorKind::Missing => "The shared folder could not be found, holds no payload, or is not public",
        ErrorKind::Network => "The Provider could not be reached, or the download was cut short",
        ErrorKind::Archive => "The download is not a payload this app can read",
        ErrorKind::Disk => "The new files could not be written to disk",
        ErrorKind::Aborted => "The download was cancelled",
    }
}

fn subtle_text<'a>(content: impl ToString) -> Element<'a, Message> {
    text(content.to_string())
        .size(BODY_SIZE)
        .style(|theme: &Theme| text::Style { color: Some(Color { a: SUBTLE_ALPHA, ..theme.palette().text }) })
        .into()
}

impl State {
    pub fn update(&mut self, message: Message, core_settings: &mut CoreSettings, held: &mut SourceState) -> Task<Message> {
        match message {
            Message::LinkChanged(link) => {
                if matches!(self.status, Status::Checking | Status::Installing(_)) {
                    return Task::none();
                }

                if held.agreement != agreement() {
                    self.pending = Some(link);
                    self.terms_open = true;

                    return Task::none();
                }

                core_settings.files.source_link = link;
                self.settle();

                Task::none()
            }
            Message::ModeSelected(mode) => {
                core_settings.files.source_mode = mode;

                Task::none()
            }
            Message::Check { manual } => {
                if !matches!(self.status, Status::Idle | Status::Current | Status::Failed(_)) {
                    return Task::none();
                }

                let link = core_settings.files.source_link.trim().to_owned();

                if link.is_empty() {
                    return Task::none();
                }

                if held.agreement != agreement() {
                    self.terms_open = manual;

                    return Task::none();
                }

                info!("Checking the game data source for newer files");
                self.settle();
                self.status = Status::Checking;

                Task::perform(
                    smol::unblock(move || {
                        let result = source::probe(&link);

                        (link, result)
                    }),
                    move |(link, result)| Message::Checked { manual, link, result },
                )
            }
            Message::Checked { manual, link, result } => {
                if !matches!(self.status, Status::Checking) {
                    return Task::none();
                }

                self.status = Status::Idle;

                if link != core_settings.files.source_link.trim() {
                    return Task::none();
                }

                match result {
                    Ok(stamp) if source::current(&stamp, core_settings.files.source_stamp.as_ref()) => {
                        info!("Game data already matches its source");

                        if manual { self.flash(Status::Current) } else { Task::none() }
                    }
                    Ok(stamp) => {
                        info!(modified = stamp.modified, "The game data source holds newer files");
                        self.status = Status::Found(summary(&stamp));
                        self.is_open = true;

                        Task::none()
                    }
                    Err(err) if manual => self.flash(Status::Failed(err.kind)),
                    Err(_) => Task::none(),
                }
            }
            Message::Show => {
                self.is_open = self.prompts();

                Task::none()
            }
            Message::Accept => {
                if !matches!(self.status, Status::Found(_)) {
                    return Task::none();
                }

                self.install(core_settings.files.source_link.trim().to_owned())
            }
            Message::Decline | Message::Acknowledge => {
                self.settle();

                Task::none()
            }
            Message::Ignore => {
                if !self.confirm.take(&()) {
                    return self.confirm.set((), Message::IgnoreExpired);
                }

                info!("User chose to ignore the game data source, changing its mode to Ignore");
                core_settings.files.source_mode = UpdateMode::Ignore;
                self.settle();

                Task::none()
            }
            Message::IgnoreExpired => {
                self.confirm.expire();

                Task::none()
            }
            Message::Cancel => {
                self.abort.store(true, Ordering::Relaxed);

                Task::none()
            }
            Message::Stepped(step) => {
                if matches!(self.status, Status::Installing(_)) {
                    self.status = Status::Installing(progress(step));
                }

                Task::none()
            }
            Message::Finished(result) => {
                watcher::resume();

                let landed = match result {
                    Ok(stamp) => {
                        core_settings.files.source_stamp = Some(stamp);
                        self.refreshed = true;

                        Status::Installed
                    }
                    Err(err) if err.kind == ErrorKind::Aborted => {
                        self.settle();

                        return Task::none();
                    }
                    Err(err) => Status::Failed(err.kind),
                };

                if self.is_open {
                    self.status = landed;

                    return Task::none();
                }

                self.flash(landed)
            }
            Message::Popup(msg) => {
                if !self.popup.update(msg, POPUP) {
                    return Task::none();
                }

                self.is_open = false;
                self.confirm.clear();

                if matches!(self.status, Status::Found(_) | Status::Installed | Status::Failed(_)) {
                    self.status = Status::Idle;
                }

                Task::none()
            }
            Message::BannerExpired => {
                self.banner.expire();

                if !self.is_open && matches!(self.status, Status::Current | Status::Installed | Status::Failed(_)) {
                    self.status = Status::Idle;
                }

                Task::none()
            }
            Message::Agree => {
                held.agreement = agreement();
                self.terms_open = false;

                if let Some(link) = self.pending.take() {
                    core_settings.files.source_link = link;
                    self.settle();
                }

                Task::none()
            }
            Message::Disagree => {
                self.terms_open = false;
                self.pending = None;
                core_settings.files.source_link.clear();
                self.settle();

                Task::none()
            }
            Message::Terms(msg) => {
                if self.terms_popup.update(msg, TERMS_POPUP) {
                    return Task::done(Message::Disagree);
                }

                Task::none()
            }
            Message::OpenUrl(url) => {
                if let Err(err) = open::that(&url) {
                    warn!("Failed to open URL {}: {}", url, err);
                }

                Task::none()
            }
        }
    }

    fn settle(&mut self) {
        self.is_open = false;
        self.confirm.clear();
        self.banner.clear();
        self.status = Status::Idle;
    }

    fn flash(&mut self, status: Status) -> Task<Message> {
        self.status = status;
        self.banner.set((), Message::BannerExpired)
    }

    fn prompts(&self) -> bool {
        matches!(self.status, Status::Found(_) | Status::Installing(_) | Status::Installed | Status::Failed(_))
    }

    fn install(&mut self, link: String) -> Task<Message> {
        watcher::suspend();
        self.abort.store(false, Ordering::Relaxed);
        self.confirm.clear();

        let (tx, rx) = mpsc::unbounded();
        let steps = tx.clone();
        let abort = self.abort.clone();

        let spawned = thread::Builder::new().name("source_install".to_owned()).spawn(move || {
            let result = source::install(
                &link,
                |step| {
                    let _ = steps.unbounded_send(Message::Stepped(step));
                },
                &abort,
            );

            if result.is_ok() {
                import::forget_imports();
            }

            let _ = tx.unbounded_send(Message::Finished(result));
        });

        if let Err(err) = spawned {
            error!("Failed to spawn the game data download thread: {}", err);
            watcher::resume();
            self.status = Status::Failed(ErrorKind::Disk);

            return Task::none();
        }

        self.status = Status::Installing(progress(Step::Downloading { done: 0, total: 0 }));

        Task::stream(rx)
    }

    pub(super) fn popup_open(&self) -> bool {
        self.is_open && self.prompts()
    }

    pub(super) fn take_refresh(&mut self) -> bool {
        std::mem::take(&mut self.refreshed)
    }

    pub(super) fn terms_open(&self) -> bool {
        self.terms_open
    }

    pub(super) fn terms_view(&self, window: Size, ui_theme: Theme) -> Option<Element<'_, Message>> {
        self.terms_open.then(|| {
            self.terms_popup.view("Acknowledgement", TERMS_POPUP, window, Message::Terms, move || self.terms_content(&ui_theme), None)
        })
    }

    fn terms_content(&self, ui_theme: &Theme) -> Element<'_, Message> {
        column![
            smooth_scroll(
                scrollable(
                    markdown::view(&self.terms, markdown::Settings::with_text_size(BODY_SIZE, theme::markdown_style(ui_theme)))
                        .map(Message::OpenUrl)
                )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .spacing(SCROLLBAR_GAP)
            ),
            Space::new().height(CHOICE_GAP),
            row![
                theme::sized_button("Agree", theme::POPUP_ACTION_BUTTON_WIDTH, theme::success_button).on_press(Message::Agree),
                theme::sized_button("Disagree", theme::POPUP_ACTION_BUTTON_WIDTH, theme::danger_button).on_press(Message::Disagree),
            ]
                .spacing(CHOICE_SPACING),
        ]
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(TERMS_PADDING)
            .into()
    }

    pub fn view<'a>(&'a self, files: &'a FilesSettings) -> Element<'a, Message> {
        let busy = matches!(self.status, Status::Checking | Status::Installing(_));

        let input = text_input("Provider link", &files.source_link)
            .on_input_maybe((!busy).then_some(Message::LinkChanged))
            .width(Length::Fixed(INPUT_WIDTH))
            .style(theme::rounded_input);

        let linked = !files.source_link.trim().is_empty();

        let (label, style, press): (&str, theme::ButtonStyleFn, Option<Message>) = match &self.status {
            Status::Idle if linked => ("Check for Files Now", theme::primary_button, Some(Message::Check { manual: true })),
            Status::Idle => ("No Source Link", theme::neutral_button, None),
            Status::Checking => ("Checking for Files...", theme::warning_status, None),
            Status::Current => ("Up to Date!", theme::success_status, None),
            Status::Found(_) => ("Files Found!", theme::success_button, Some(Message::Show)),
            Status::Installing(_) => ("Downloading Files...", theme::warning_button, Some(Message::Show)),
            Status::Installed => ("Files Updated!", theme::success_status, None),
            Status::Failed(ErrorKind::Link) => ("Invalid Link!", theme::danger_status, None),
            Status::Failed(ErrorKind::Missing) => ("Folder Not Found!", theme::danger_status, None),
            Status::Failed(ErrorKind::Network) => ("Failed to Check!", theme::danger_status, None),
            Status::Failed(_) => ("Download Failed!", theme::danger_status, None),
        };

        let selected = match files.source_mode {
            UpdateMode::Prompt => MODES[0],
            UpdateMode::Ignore => MODES[1],
        };

        let mode = row![
            hover_hint(text("File Handling"), MODE_HINT),
            pick_list(MODES, Some(selected), |picked| {
                Message::ModeSelected(if picked == MODES[0] { UpdateMode::Prompt } else { UpdateMode::Ignore })
            })
            .style(theme::combo_box)
            .menu_style(theme::combo_box_menu),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        column![
            hover_hint(input, LINK_HINT),
            theme::sized_button(label, theme::STATUS_BUTTON_WIDTH, style).on_press_maybe(press),
            mode,
        ]
        .spacing(10)
        .into()
    }

    pub(super) fn popup_view(&self, window: Size) -> Option<Element<'_, Message>> {
        let title = match &self.status {
            Status::Found(_) => "New Files Available",
            Status::Installing(_) => "Downloading Files",
            Status::Installed => "Files Updated",
            Status::Failed(_) => "Download Failed",
            Status::Idle | Status::Checking | Status::Current => return None,
        };

        Some(self.popup.view(title, POPUP, window, Message::Popup, move || {
            let body = match &self.status {
                Status::Found(summary) => self.view_found(summary),
                Status::Installing(progress) => view_installing(progress),
                Status::Installed => view_installed(),
                Status::Failed(kind) => view_failed(*kind),
                Status::Idle | Status::Checking | Status::Current => Space::new().into(),
            };

            container(body)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(POPUP_PADDING)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into()
        }, None))
    }

    fn view_found<'a>(&'a self, summary: &'a str) -> Element<'a, Message> {
        let ignore_label = if self.confirm.is_set() { CONFIRM_SHORT_LABEL } else { "Ignore" };

        let actions = row![
            theme::sized_button("Yes", theme::POPUP_ACTION_BUTTON_WIDTH, theme::success_button).on_press(Message::Accept),
            theme::sized_button("No", theme::POPUP_ACTION_BUTTON_WIDTH, theme::neutral_button).on_press(Message::Decline),
            theme::sized_button(ignore_label, theme::POPUP_ACTION_BUTTON_WIDTH, theme::danger_button).on_press(Message::Ignore),
        ]
        .spacing(BUTTON_SPACING);

        column![
            theme::bold_text("New game files are available").size(TITLE_SIZE),
            subtle_text(summary),
            text("Would you like to download them now?").size(BODY_SIZE),
            actions,
            subtle_text("Downloading replaces your \"game\" folder"),
            subtle_text("\"Ignore\" stops all future file checks on launch"),
        ]
        .spacing(CONTENT_SPACING)
        .align_x(Alignment::Center)
        .into()
    }
}

fn view_installing<'a>(progress: &'a Progress) -> Element<'a, Message> {
    column![
        theme::bold_text(progress.heading).size(TITLE_SIZE),
        progress_bar(0.0..=1.0, progress.fraction)
            .length(Length::Fixed(PROGRESS_WIDTH))
            .girth(Length::Fixed(PROGRESS_HEIGHT))
            .style(theme::progress_track),
        subtle_text(&progress.detail),
        theme::sized_button("Cancel", theme::POPUP_ACTION_BUTTON_WIDTH, theme::danger_button)
            .on_press_maybe((!progress.sealed).then_some(Message::Cancel)),
    ]
    .spacing(CONTENT_SPACING)
    .align_x(Alignment::Center)
    .into()
}

fn view_installed<'a>() -> Element<'a, Message> {
    column![
        theme::bold_text("The new game files are installed").size(TITLE_SIZE),
        subtle_text("Everything is being reloaded from them now"),
        theme::sized_button("Okay", theme::POPUP_ACTION_BUTTON_WIDTH, theme::success_button).on_press(Message::Acknowledge),
    ]
    .spacing(CONTENT_SPACING)
    .align_x(Alignment::Center)
    .into()
}

fn view_failed<'a>(kind: ErrorKind) -> Element<'a, Message> {
    column![
        theme::bold_text("The new game files were not installed").size(TITLE_SIZE),
        text(failure(kind)).size(BODY_SIZE),
        subtle_text("Your current \"game\" folder was left as it was"),
        theme::sized_button("Okay", theme::POPUP_ACTION_BUTTON_WIDTH, theme::neutral_button).on_press(Message::Acknowledge),
    ]
    .spacing(CONTENT_SPACING)
    .align_x(Alignment::Center)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_upload_date_drops_its_weekday_and_clock() {
        assert_eq!(uploaded("Thu, 20 May 2021 11:16:29 GMT"), "20 May 2021");
        assert_eq!(uploaded("20 May 2021"), "20 May 2021");
        assert_eq!(uploaded(""), "");
    }

    // Nothing typed may reach the settings until the agreement is on record.
    #[test]
    fn a_link_only_lands_once_the_agreement_is_accepted() {
        let mut state = State::default();
        let mut settings = CoreSettings::default();
        let mut held = SourceState::default();

        let _ = state.update(Message::LinkChanged("first".to_owned()), &mut settings, &mut held);
        assert!(state.terms_open() && settings.files.source_link.is_empty());

        let _ = state.update(Message::Disagree, &mut settings, &mut held);
        assert!(!state.terms_open() && settings.files.source_link.is_empty() && held.agreement.is_empty());

        let _ = state.update(Message::LinkChanged("second".to_owned()), &mut settings, &mut held);
        let _ = state.update(Message::Agree, &mut settings, &mut held);
        assert_eq!(settings.files.source_link, "second");
        assert_eq!(held.agreement, agreement());

        let _ = state.update(Message::LinkChanged("third".to_owned()), &mut settings, &mut held);
        assert!(!state.terms_open());
        assert_eq!(settings.files.source_link, "third");

        // A link saved before the agreement existed is wiped by a refusal too.
        held.agreement.clear();
        let _ = state.update(Message::Check { manual: true }, &mut settings, &mut held);
        assert!(state.terms_open());
        let _ = state.update(Message::Disagree, &mut settings, &mut held);
        assert!(settings.files.source_link.is_empty());
    }
}
