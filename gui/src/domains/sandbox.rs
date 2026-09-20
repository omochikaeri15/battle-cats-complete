use iced::widget::{column, markdown, row, scrollable, Space};
use iced::{Alignment, Element, Length, Size, Task, Theme};
use tracing::warn;

use std::cell::RefCell;
use std::rc::Rc;

use kore::Vfs;

use crate::app::state::AppState;
use crate::app::theme;
use crate::systems::emu::{Frame as EmuFrame, Session};
use crate::systems::emu::SheetCache;
use crate::widget::{popup, smooth_scroll};

const ACKNOWLEDGEMENT: &str = r#"
The purpose of this agreement is to ensure that you, the User, are aware of the potential quirks regarding Sandbox.
By accepting this agreement, you must have read, understood, and accepted the following facts about Sandbox:

- Sandbox may have known but subtle inaccuracies, mechanical bugs, or logical quirks when compared to the real game.
- Sandbox may not be updated to support features and/or mechanics from the latest game version(s) within a timely manner.
- Sandbox is representative of the vanilla game logic and it's capabilities only, with no custom functionality added or planned.
- Sandbox is not an avenue that replaces the appeal and purpose of the game in any capacity, whether that be vanilla or modded.

You can accept the agreement by clicking the "Agree" button below. Selecting "Disagree" will redirect you to the Home menu.
"#;
const ACKNOWLEDGE_POPUP: popup::Spec = popup::Spec::new(popup::Kind::Acknowledgement, Size::new(560.0, 435.0));
const BODY_SIZE: f32 = 14.0;
const BODY_PADDING: f32 = 20.0;
const SCROLLBAR_GAP: f32 = 8.0;
const CHOICE_SPACING: f32 = 12.0;
const CHOICE_GAP: f32 = 18.0;

#[derive(Debug, Clone)]
pub enum Message {
    Popup(popup::Message),
    OpenUrl(String),
    Agree,
    Disagree,
    Play,
    Tick,
}

pub struct State {
    prompt_open: bool,
    prompt: popup::State,
    terms: Vec<markdown::Item>,
    status: String,
    session: Option<Session>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            prompt_open: false,
            prompt: popup::State::default(),
            terms: crate::common::markdown::parse(ACKNOWLEDGEMENT),
            status: String::new(),
            session: None,
        }
    }
}

impl State {
    pub fn enter(&mut self, acknowledged: bool) {
        self.prompt_open = !acknowledged;
    }

    pub fn transitioning(&self) -> bool {
        self.session.as_ref().is_some_and(Session::running)
    }

    pub fn curtain_frame(&self) -> Option<&Rc<RefCell<EmuFrame>>> {
        self.session.as_ref().map(Session::frame)
    }

    pub fn curtain_covered(&self) -> bool {
        self.session.as_ref().is_some_and(Session::covered)
    }

    pub fn curtain_sheets(&self) -> Option<&Rc<RefCell<SheetCache>>> {
        self.session.as_ref().map(Session::sheets)
    }

    pub fn design_height(&self) -> f32 {
        self.session.as_ref().map_or(0.0, Session::design_height)
    }

    pub fn letterbox_shift(&self) -> f32 {
        self.session.as_ref().map_or(0.0, Session::letterbox_shift)
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        if let Some(session) = self.session.as_mut() {
            session.resize(width, height);
        }
    }

    pub fn start(&mut self, width: f32, height: f32) {
        let session = self.session.get_or_insert_with(Session::new);

        session.resize(width, height);
        session.begin();
        self.status = String::new();
    }

    pub fn update(&mut self, message: Message, app_state: &mut AppState, vfs: &Vfs) -> Task<Message> {
        match message {
            Message::Popup(msg) => {
                if self.prompt.update(msg, ACKNOWLEDGE_POPUP) {
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
            Message::Agree => {
                app_state.sandbox.acknowledged = true;
                self.prompt_open = false;

                Task::none()
            }
            Message::Disagree => {
                self.prompt_open = false;

                Task::none()
            }
            Message::Play => Task::none(),
            Message::Tick => {
                if let Some(session) = self.session.as_mut() {
                    session.tick(vfs);

                    let labels = session.frame().borrow().labels;

                    self.status = match session.failure() {
                        Some(reason) => reason.to_owned(),
                        None if labels > 0 => format!("{labels} text labels are not drawn yet"),
                        None => String::new(),
                    };
                }

                Task::none()
            }
        }
    }

    pub fn prompt_open(&self) -> bool {
        self.prompt_open
    }

    pub fn prompt_view(&self, window: Size, ui_theme: Theme) -> Option<Element<'_, Message>> {
        self.prompt_open.then(|| {
            self.prompt.view("Acknowledgement", ACKNOWLEDGE_POPUP, window, Message::Popup, move || self.prompt_content(&ui_theme), None)
        })
    }

    fn prompt_content(&self, ui_theme: &Theme) -> Element<'_, Message> {
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
            .padding(BODY_PADDING)
            .into()
    }

    pub fn view(&self) -> Element<'_, Message> {
        column![
            theme::sized_button("Play", theme::POPUP_ACTION_BUTTON_WIDTH, theme::success_button)
                .on_press(Message::Play),
            Space::new().height(CHOICE_GAP),
            iced::widget::text(self.status.as_str()).size(BODY_SIZE),
        ]
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(BODY_PADDING)
            .into()
    }
}
