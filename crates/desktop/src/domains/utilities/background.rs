use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use iced::alignment::Horizontal;
use iced::widget::{column, container, row, text_input};
use iced::{Element, Length, Task};
use codex::chapter::stage::Background;
use tracing::warn;

use kokoro::domains::utilities::background::{self as builder, Catalog, Loaded, Names, Overrides, DEFAULT_SEED};
use kokoro::Vault;

use crate::app::state::BackgroundState;
use crate::app::theme;
use crate::common::dialog;
use crate::systems::background as viewer;
use crate::widget::{hover_hint, picker};

const PANEL_PADDING: f32 = 12.0;
const ROW_GAP: f32 = 8.0;
const ID_INPUT_WIDTH: f32 = picker::BUTTON_WIDTH * 2.0 + ROW_GAP;
const SEED_INPUT_WIDTH: f32 = picker::BUTTON_WIDTH;
const INPUT_PADDING: [f32; 2] = [6.0, 10.0];
const SEED_CEILING: u64 = i32::MAX as u64;

const NO_BACKGROUNDS_HINT: &str = "No backgrounds within mount";
const VANILLA_HINT: &str = "Vanilla background loaded\nPlease clear ID field first";
const ID_HINT: &str = "ID...";
const SEED_HINT: &str = "Seed...";
const RANGES_PER_LINE: usize = 6;
const TABLE_PREFIX: &str = "bg";

#[derive(Clone)]
pub enum Message {
    PickPng,
    PickImgcut,
    PickCsv,
    PngPicked(Option<PathBuf>),
    ImgcutPicked(Option<PathBuf>),
    CsvPicked(Option<PathBuf>),
    IdChanged(String),
    SeedChanged(String),
    Scanned(u64, Catalog),
    Loaded(u64, Result<Arc<Loaded>, String>),
    Viewer(viewer::Message),
    Tick,
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PickPng => write!(f, "PickPng"),
            Self::PickImgcut => write!(f, "PickImgcut"),
            Self::PickCsv => write!(f, "PickCsv"),
            Self::PngPicked(path) => write!(f, "PngPicked({path:?})"),
            Self::ImgcutPicked(path) => write!(f, "ImgcutPicked({path:?})"),
            Self::CsvPicked(path) => write!(f, "CsvPicked({path:?})"),
            Self::IdChanged(value) => write!(f, "IdChanged({value:?})"),
            Self::SeedChanged(value) => write!(f, "SeedChanged({value:?})"),
            Self::Scanned(generation, catalog) => write!(f, "Scanned({generation}, {} ids)", catalog.ids.len()),
            Self::Loaded(generation, outcome) => write!(f, "Loaded({generation}, {})", outcome.is_ok()),
            Self::Viewer(msg) => write!(f, "Viewer({msg:?})"),
            Self::Tick => write!(f, "Tick"),
        }
    }
}

#[derive(Default)]
pub struct State {
    overrides: Overrides,
    names: Option<Names>,
    id_input: String,
    seed_input: String,
    catalog: Catalog,
    ranges: String,
    scanned: bool,
    scanning: bool,
    generation: u64,
    loading: Option<u64>,
    notice: Option<String>,
    watched: BTreeSet<String>,
    viewer: viewer::State,
}

fn is_background_file(name: &str) -> bool {
    name.to_ascii_lowercase().starts_with(TABLE_PREFIX)
}

impl State {
    pub(super) fn pace(&self) -> std::time::Duration {
        self.viewer.pace()
    }

    fn seed(&self) -> u32 {
        self.seed_input.parse::<u32>().ok().filter(|seed| *seed > 0).unwrap_or(DEFAULT_SEED)
    }

    fn id(&self) -> Option<u32> {
        self.id_input.parse::<u32>().ok()
    }

    fn vanilla(&self) -> bool {
        !self.id_input.is_empty() && self.overrides.csv.is_none()
    }

    fn next_generation(&mut self) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.generation
    }

    pub fn update(&mut self, message: Message, vault: &Arc<Vault>, prefs: &mut BackgroundState) -> Task<Message> {
        match message {
            Message::PickPng => Task::perform(dialog::file("PNG Image", &["png"]), Message::PngPicked),
            Message::PickImgcut => Task::perform(dialog::file("Sprite Cut List", &["imgcut"]), Message::ImgcutPicked),
            Message::PickCsv => Task::perform(dialog::file("Background Table", &["csv"]), Message::CsvPicked),
            Message::PngPicked(Some(path)) => {
                self.overrides.png = Some(path);
                self.load(vault)
            }
            Message::ImgcutPicked(Some(path)) => {
                self.overrides.imgcut = Some(path);
                self.load(vault)
            }
            Message::CsvPicked(Some(path)) => {
                self.overrides.csv = Some(path);
                self.scanned = false;
                Task::batch([self.scan(vault), self.load(vault)])
            }
            Message::PngPicked(None) | Message::ImgcutPicked(None) | Message::CsvPicked(None) => Task::none(),
            Message::IdChanged(value) => {
                if !value.chars().all(|c| c.is_ascii_digit()) {
                    return Task::none();
                }

                self.id_input = value;
                self.load(vault)
            }
            Message::SeedChanged(value) => {
                if !value.chars().all(|c| c.is_ascii_digit()) {
                    return Task::none();
                }

                if !value.is_empty() && !value.parse::<u64>().is_ok_and(|seed| (1..=SEED_CEILING).contains(&seed)) {
                    return Task::none();
                }

                self.seed_input = value;
                self.viewer.set_seed(self.seed());
                Task::none()
            }
            Message::Scanned(generation, catalog) => {
                if generation != self.generation {
                    return Task::none();
                }

                self.scanning = false;
                self.scanned = true;
                self.ranges = wrapped_ranges(&catalog);
                self.catalog = catalog;
                Task::none()
            }
            Message::Loaded(generation, outcome) => {
                if self.loading != Some(generation) {
                    return Task::none();
                }

                self.loading = None;

                match outcome {
                    Ok(loaded) => {
                        let loaded = Arc::unwrap_or_clone(loaded);

                        self.names = Some(loaded.names.clone());
                        self.watched = loaded.watched.clone();
                        self.notice = None;
                        self.viewer.adopt(loaded, self.seed(), prefs);
                    }
                    Err(problem) => {
                        self.names = None;
                        self.notice = Some(problem);
                        self.viewer.clear();
                    }
                }

                Task::none()
            }
            Message::Tick => {
                let scan = if self.scanned || self.scanning { Task::none() } else { self.scan(vault) };

                self.viewer.tick();
                scan
            }
            Message::Viewer(msg) => self.viewer.update(msg, prefs).map(Message::Viewer),
        }
    }

    fn scan(&mut self, vault: &Arc<Vault>) -> Task<Message> {
        let generation = self.next_generation();
        let vault = Arc::clone(vault);
        let csv = self.overrides.csv.clone();

        self.scanning = true;

        Task::perform(
            smol::unblock(move || match csv {
                Some(path) => std::fs::read(&path)
                    .inspect_err(|err| warn!(path = %path.display(), "Background table could not be read: {}", err))
                    .ok()
                    .and_then(|bytes| Background::parse(bytes, None).ok())
                    .map(|table| Catalog::from_table(&table))
                    .unwrap_or_default(),
                None => Catalog::scan(&vault.vfs),
            }),
            move |catalog| Message::Scanned(generation, catalog),
        )
    }

    fn load(&mut self, vault: &Arc<Vault>) -> Task<Message> {
        let Some(id) = self.id() else {
            self.loading = None;
            self.names = None;
            self.notice = None;
            self.viewer.clear();
            return Task::none();
        };
        let generation = self.next_generation();
        let vault = Arc::clone(vault);
        let overrides = if self.vanilla() { Overrides::default() } else { self.overrides.clone() };

        self.loading = Some(generation);

        Task::perform(
            smol::unblock(move || builder::load(&vault.vfs, id, &overrides).map(Arc::new)),
            move |outcome| Message::Loaded(generation, outcome),
        )
    }

    pub(crate) fn files_changed(&mut self, paths: &[PathBuf], vault: &Arc<Vault>) -> Task<Message> {
        let touched = paths.iter().filter_map(|path| path.file_name().and_then(|name| name.to_str())).any(|name| {
            is_background_file(name) || self.watched.contains(&name.to_ascii_lowercase())
        });

        if !touched {
            return Task::none();
        }

        self.invalidate(vault)
    }

    pub(crate) fn invalidate(&mut self, vault: &Arc<Vault>) -> Task<Message> {
        self.scanned = false;

        Task::batch([self.scan(vault), self.load(vault)])
    }

    pub fn view<'a>(&'a self, prefs: &'a BackgroundState) -> Element<'a, Message> {
        column![self.view_controls(), self.view_viewer(prefs)].spacing(ROW_GAP).height(Length::Fill).into()
    }

    fn view_controls(&self) -> Element<'_, Message> {
        let names = self.names.as_ref();
        let vanilla = self.vanilla();
        let slot = |label: &'static str, picked: Option<&PathBuf>, name: Option<&str>, press: Message| -> Element<'_, Message> {
            if vanilla {
                let shown = name.map(Path::new);

                return hover_hint(picker::slot(label, shown, press).on_press_maybe(None), VANILLA_HINT);
            }

            picker::slot(label, picked.map(PathBuf::as_path), press).into()
        };
        let files = row![
            slot("Add PNG", self.overrides.png.as_ref(), names.map(|names| names.png.as_str()), Message::PickPng),
            slot("Add IMGCUT", self.overrides.imgcut.as_ref(), names.map(|names| names.imgcut.as_str()), Message::PickImgcut),
            slot("Add BG CSV", self.overrides.csv.as_ref(), names.map(|names| names.csv.as_str()), Message::PickCsv),
        ]
        .spacing(ROW_GAP);

        let available = !self.catalog.ids.is_empty();
        let hint = if available { self.ranges.as_str() } else { NO_BACKGROUNDS_HINT };
        let id_input = text_input(ID_HINT, &self.id_input)
            .on_input_maybe(available.then_some(Message::IdChanged))
            .width(Length::Fixed(ID_INPUT_WIDTH))
            .padding(INPUT_PADDING)
            .size(picker::TEXT_SIZE)
            .style(theme::rounded_input);
        let id_input = hover_hint(id_input, hint);
        let seed_input = text_input(SEED_HINT, &self.seed_input)
            .on_input(Message::SeedChanged)
            .width(Length::Fixed(SEED_INPUT_WIDTH))
            .padding(INPUT_PADDING)
            .size(picker::TEXT_SIZE)
            .style(theme::rounded_input);
        let inputs = row![id_input, seed_input].spacing(ROW_GAP);

        let mut body = column![centered(files), centered(inputs)].spacing(ROW_GAP);

        if let Some(notice) = &self.notice {
            body = body.push(centered(theme::centered_text(notice.as_str()).size(picker::TEXT_SIZE - 1.0)));
        }

        container(body).padding(PANEL_PADDING).into()
    }

    fn view_viewer<'a>(&'a self, prefs: &'a BackgroundState) -> Element<'a, Message> {
        container(self.viewer.view(prefs).map(Message::Viewer)).width(Length::Fill).height(Length::Fill).into()
    }
}

fn wrapped_ranges(catalog: &Catalog) -> String {
    let ranges = catalog.ranges();
    let spans: Vec<&str> = ranges.split(", ").collect();

    spans.chunks(RANGES_PER_LINE).map(|line| line.join(", ")).collect::<Vec<String>>().join("\n")
}

fn centered<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content).width(Length::Fill).align_x(Horizontal::Center).into()
}
