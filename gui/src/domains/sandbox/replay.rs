use std::path::{Path, PathBuf};

use iced::widget::{column, container, row, scrollable, text, Column};
use iced::{Color, Element, Length, Task};

use emu::runtime::VERSION;
use kore::domains::sandbox::replay::{self as tape, Summary};

use crate::app::theme;
use crate::systems::emu::Reel;
use crate::widget::{list_row, section, smooth_scroll};

const LIST_WIDTH: f32 = 220.0;
const PANEL_PADDING: f32 = 20.0;
const ROW_SPACING: f32 = 8.0;
const LIST_SPACING: f32 = 4.0;
const LABEL_SIZE: f32 = 14.0;
const LABEL_WIDTH: f32 = 110.0;
const SAVE_WIDTH: f32 = 160.0;
const LATEST_LABEL: &str = "Latest Battle";
const LATEST_COLOR: Color = Color::from_rgb(0.35, 0.63, 0.92);
const FRAMES_PER_SECOND: usize = 30;
const MEGABYTE: f64 = 1024.0 * 1024.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pick {
    Latest,
    Bundle(PathBuf),
}

#[derive(Clone, Debug)]
pub enum Message {
    Select(Pick),
    Save,
    Saved(Result<PathBuf, String>),
}

struct Details {
    title: String,
    version: Option<String>,
    rows: Vec<(&'static str, String)>,
    ready: bool,
}

#[derive(Default)]
pub struct State {
    bundles: Vec<PathBuf>,
    picked: Option<Pick>,
    details: Option<Details>,
    saving: bool,
    note: String,
}

fn bundle_name(path: &Path) -> String {
    path.file_stem().map_or_else(String::new, |stem| stem.to_string_lossy().into_owned())
}

fn clock(frames: usize) -> String {
    let seconds = frames / FRAMES_PER_SECOND;

    format!("{frames} frames ({}:{:02})", seconds / 60, seconds % 60)
}

fn describe(title: String, summary: &Summary, units: &dyn Fn(u32, usize) -> String) -> Details {
    let mut rows = Vec::new();

    match &summary.save {
        Some(save) => {
            let stage = &save.setup.stage;
            let version = if save.version.is_empty() { "Unknown" } else { save.version.as_str() };

            rows.push(("Version", version.to_owned()));

            rows.push(("Stage", format!("Map {} · Stage {}", stage.map_id, stage.stage)));
            rows.push(("Crowns", (stage.crown + 1).to_string()));

            for (slot, unit) in save.setup.lineup.iter().enumerate() {
                let label = if slot == 0 { "Lineup" } else { "" };
                let form = usize::try_from(unit.form).unwrap_or(0);
                let id = u32::try_from(unit.unit).unwrap_or(0);

                rows.push((label, format!("{} · Lv {}+{}", units(id, form), unit.level, unit.plus)));
            }

            rows.push(("Screen", format!("{} × {}{}", save.screen.width, save.screen.height, if save.screen.phone { " (phone)" } else { "" })));
        }
        None => rows.push(("Save", "missing".to_owned())),
    }

    rows.push(("Input", summary.frames.map_or_else(|| "missing".to_owned(), clock)));
    rows.push(("Assets", format!("{} files", summary.assets)));
    rows.push(("Size", format!("{:.1} MB", summary.bytes as f64 / MEGABYTE)));

    Details {
        title,
        version: summary.save.as_ref().map(|save| save.version.clone()),
        rows,
        ready: summary.save.is_some() && summary.frames.is_some(),
    }
}

impl State {
    pub fn refresh(&mut self, units: &dyn Fn(u32, usize) -> String) {
        self.bundles = tape::list(&tape::library());

        let picked = match self.picked.take() {
            Some(Pick::Bundle(path)) if !self.bundles.contains(&path) => None,
            other => other,
        };

        self.select(picked.unwrap_or(Pick::Latest), units);
    }

    fn select(&mut self, pick: Pick, units: &dyn Fn(u32, usize) -> String) {
        let (title, summary) = match &pick {
            Pick::Latest => (LATEST_LABEL.to_owned(), tape::scratch().map(|dir| tape::inspect_dir(&dir)).unwrap_or_default()),
            Pick::Bundle(path) => (bundle_name(path), tape::inspect_bundle(path)),
        };

        self.details = Some(describe(title, &summary, units));
        self.picked = Some(pick);
    }

    pub fn foreign_version(&self) -> Option<&str> {
        let recorded = self.details.as_ref()?.version.as_deref()?;

        (recorded != VERSION).then_some(recorded)
    }

    pub fn target(&self) -> Option<PathBuf> {
        match self.picked.as_ref()? {
            Pick::Latest => tape::scratch(),
            Pick::Bundle(path) => Some(path.clone()),
        }
    }

    pub fn reel(&self) -> Option<Reel> {
        if !self.details.as_ref().is_some_and(|details| details.ready) {
            return None;
        }

        match self.picked.as_ref()? {
            Pick::Latest => Some(Reel::Latest),
            Pick::Bundle(path) => Some(Reel::Bundle(path.clone())),
        }
    }

    pub fn update(&mut self, message: Message, units: &dyn Fn(u32, usize) -> String) -> Task<Message> {
        match message {
            Message::Select(pick) => {
                self.note.clear();
                self.select(pick, units);

                Task::none()
            }
            Message::Save => {
                if self.saving {
                    return Task::none();
                }

                let Some(dir) = tape::scratch() else {
                    self.note = "There is no state folder holding the latest battle".to_owned();

                    return Task::none();
                };
                let out = tape::next_bundle(&tape::library());

                self.saving = true;
                self.note = "Saving…".to_owned();

                Task::perform(smol::unblock(move || tape::pack(&dir, &out).map(|()| out)), Message::Saved)
            }
            Message::Saved(outcome) => {
                self.saving = false;

                match outcome {
                    Ok(path) => {
                        self.note = format!("Saved as {}", bundle_name(&path));
                        self.refresh(units);
                    }
                    Err(reason) => self.note = format!("Could not save: {reason}"),
                }

                Task::none()
            }
        }
    }

    pub fn view(&self, bottom: f32) -> Element<'_, Message> {
        let latest_picked = self.picked == Some(Pick::Latest);
        let latest_label = text(LATEST_LABEL).size(LABEL_SIZE);
        let latest_label = if latest_picked { latest_label } else { latest_label.color(LATEST_COLOR) };
        let mut list = column![list_row(
            container(latest_label).padding([8, 12]).width(Length::Fill),
            latest_picked,
            true,
            Length::Fill,
            Message::Select(Pick::Latest),
        )]
            .spacing(LIST_SPACING);

        for path in &self.bundles {
            let picked = matches!(&self.picked, Some(Pick::Bundle(held)) if held == path);

            list = list.push(list_row(
                container(theme::button_label(bundle_name(path)).size(LABEL_SIZE)).padding([8, 12]).width(Length::Fill),
                picked,
                true,
                Length::Fill,
                Message::Select(Pick::Bundle(path.clone())),
            ));
        }

        let sidebar = container(smooth_scroll(scrollable(list).height(Length::Fill)))
            .width(Length::Fixed(LIST_WIDTH))
            .height(Length::Fill)
            .padding(8)
            .style(theme::list_panel_container);

        let body: Element<'_, Message> = match &self.details {
            Some(details) => {
                let mut rows = Column::new().spacing(ROW_SPACING);

                for (label, value) in &details.rows {
                    rows = rows.push(row![
                        text(*label).size(LABEL_SIZE).width(Length::Fixed(LABEL_WIDTH)),
                        text(value.as_str()).size(LABEL_SIZE),
                    ]);
                }

                let mut content = column![section(details.title.as_str(), Length::Fill, rows)].spacing(ROW_SPACING * 2.0);

                if latest_picked {
                    content = content.push(
                        theme::sized_button("Save Replay", SAVE_WIDTH, theme::primary_button)
                            .on_press_maybe((details.ready && !self.saving).then_some(Message::Save)),
                    );
                }

                if !self.note.is_empty() {
                    content = content.push(text(self.note.as_str()).size(LABEL_SIZE));
                }

                content.into()
            }
            None => text("Select a replay").size(LABEL_SIZE).into(),
        };

        let page = container(body).padding(iced::Padding { bottom, ..iced::Padding::new(PANEL_PADDING) }).width(Length::Fill);

        row![sidebar, smooth_scroll(scrollable(page).width(Length::Fill).height(Length::Fill))]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
