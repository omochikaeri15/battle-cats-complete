mod manage;

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::alignment::{Horizontal, Vertical};
use iced::widget::image::Handle;
use iced::widget::{button, column, container, image as iced_image, mouse_area, row, rule, scrollable, stack, text, text_input, Column, Row, Space};
use iced::{mouse, Border, Color, Element, Length, Padding, Size, Task, Theme};
use tracing::warn;

use emu::runtime::VERSION;
use kore::common::context::GlobalContext;
use kore::common::frames;
use kore::domains::cat::scanner;
use kore::domains::sandbox::orb::Allowance;
use kore::domains::sandbox::replay::{self as tape, Staged, Summary, Unit};
use kore::domains::settings::{ScannerConfig, Settings};
use kore::Vault;

use crate::app::state::AppState;
use crate::app::theme;
use crate::common::{fonts, header_icon};
use crate::domains::cat;
use crate::domains::stage::CROWN_GLYPH;
use crate::systems::emu::Reel;
use crate::widget::{list_row, popup, section, smooth_scroll, text_with_superscript};

use super::lineup::{Metrics, SMALLEST};
use super::orbs;

const SCOPE: &str = "sandbox-replay-units";
const UNIT_POPUP: popup::Spec = popup::Spec::new(popup::Kind::ReplayUnit, Size::new(760.0, 540.0));
const ROW_SPACING: f32 = 8.0;
const LIST_SPACING: f32 = 4.0;
const LABEL_SIZE: f32 = 14.0;
const LATEST_LABEL: &str = "Latest Battle";
const LIST_WIDTH: f32 = 200.0;
const LIST_PADDING: f32 = 8.0;
const ROW_HEIGHT: f32 = 40.0;
const ROW_TEXT_SIZE: f32 = 12.0;
const EDGE_GAP: f32 = 4.0;
const SCROLLBAR_GAP: f32 = 2.0;
const SEARCH_GAP: f32 = 4.0;
const LIST_GAP: f32 = 8.0;
const TITLE_SIZE: f32 = 22.0;
const TITLE_PADDING: f32 = 8.0;
const RULE_THICKNESS: f32 = 1.0;
const RULE_PADDING: f32 = 8.0;
const SECTION_GAP: f32 = 16.0;
const DETAILS_PADDING: f32 = 16.0;
const FIELD_LABEL_WIDTH: f32 = 90.0;
const CONTENT_WIDTH: f32 = 515.0;
const CARD_PADDING: f32 = 12.0;
const MEGABYTE: f64 = 1024.0 * 1024.0;
const COLUMNS: usize = 5;
const LINES: usize = 2;
const BADGE_SIZE: f32 = 12.0;
const BADGE_FILL: f32 = 0.62;
const EMPTY_FILL: f32 = 0.07;
const EDGE_ALPHA: f32 = 0.25;
const ORB_SIZE: f32 = 26.0;
const ORB_STEP: f32 = 0.55;
const COIN_SIZE: f32 = 18.0;
const SLOT_ORB_SIZE: f32 = 44.0;
const ORB_HEADER_SIZE: f32 = 18.0;
const FIRST_TALENT_FORM: usize = 2;
const DOUBLE_CLICK: Duration = Duration::from_millis(400);
const NP_ICON: &str = "gatyaitemD_07_f.png";
const FORM_LETTERS: [char; 4] = ['f', 'c', 's', 'u'];

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Pick {
    Latest,
    Bundle(PathBuf),
}

#[derive(Clone)]
pub enum Message {
    Select(Pick),
    Search(String),
    RenameInput(String),
    CommitRename,
    Press(usize),
    Cat(cat::Message),
    Orbs(orbs::Message),
    UnitPopup(popup::Message),
    Manage(manage::Message),
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Select(pick) => write!(f, "Select({pick:?})"),
            Self::Search(query) => write!(f, "Search({query})"),
            Self::RenameInput(name) => write!(f, "RenameInput({name})"),
            Self::CommitRename => write!(f, "CommitRename"),
            Self::Press(slot) => write!(f, "Press({slot})"),
            Self::Cat(msg) => write!(f, "Cat({msg:?})"),
            Self::Orbs(msg) => write!(f, "Orbs({msg:?})"),
            Self::UnitPopup(_) => write!(f, "UnitPopup"),
            Self::Manage(msg) => write!(f, "Manage({msg:?})"),
        }
    }
}

struct Tile {
    id: u32,
    form: usize,
    level: String,
    cost: Option<String>,
    inspected: String,
    talents: HashMap<u8, u8>,
    talented: bool,
    orbs: Vec<u32>,
}

struct Details {
    title: String,
    meta: Vec<(&'static str, String)>,
    game: Vec<(&'static str, String)>,
    stage: Option<(String, i32)>,
    version: Option<String>,
    ready: bool,
}

#[derive(Default)]
pub struct State {
    bundles: Vec<PathBuf>,
    picked: Option<Pick>,
    details: Option<Details>,
    staged: Option<Arc<Staged>>,
    failure: Option<String>,
    cache: Vec<(Pick, String, Arc<Staged>)>,
    metrics: Option<Metrics>,
    tiles: Vec<Tile>,
    icons: HashMap<(u32, usize), Handle>,
    decoded: header_icon::Cache,
    coin: Option<Handle>,
    inspector: cat::State,
    orbs: orbs::State,
    unit_popup: popup::State,
    open: Option<usize>,
    clicked: Option<(usize, Instant)>,
    search: String,
    names: Vec<String>,
    shown: Vec<usize>,
    rename: String,
    manage: manage::State,
}

pub fn is_bundle(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case(tape::EXTENSION))
        && path.parent().is_some_and(|parent| parent.file_name().is_some_and(|name| name == tape::library().as_os_str()))
}

fn bundle_name(path: &Path) -> String {
    path.file_stem().map_or_else(String::new, |stem| stem.to_string_lossy().into_owned())
}

fn clamped(level: u32, plus: u32, altar_cap: Option<i32>) -> (u32, u32) {
    match altar_cap {
        Some(cap) => (u32::try_from(cap).map_or(level, |cap| level.min(cap.saturating_add(1))), 0),
        None => (level, plus),
    }
}

fn level_label((level, plus): (u32, u32)) -> String {
    if plus == 0 { format!("Lv{level}") } else { format!("Lv{level}+{plus}") }
}

fn target_of(pick: &Pick) -> Option<PathBuf> {
    match pick {
        Pick::Latest => tape::scratch(),
        Pick::Bundle(path) => Some(path.clone()),
    }
}

fn describe(title: String, summary: &Summary) -> Details {
    let mut meta = Vec::new();
    let mut game = Vec::new();
    let mut stage = None;

    meta.push(("Time", summary.frames.map_or_else(|| "Missing".to_owned(), |count| frames::label(i32::try_from(count).unwrap_or(i32::MAX)))));

    if let Some(save) = &summary.save {
        let phone = if save.screen.phone { " (Phone)" } else { "" };

        meta.push(("Screen", format!("{} × {}{phone}", save.screen.width, save.screen.height)));
    }

    meta.push(("Assets", format!("{} files", summary.assets)));
    meta.push(("Size", format!("{:.1} MB", summary.bytes as f64 / MEGABYTE)));

    match &summary.save {
        Some(save) => {
            let entry = &save.setup.stage;
            let map = if entry.map_name.is_empty() { format!("Map {}", entry.map_id) } else { entry.map_name.clone() };
            let named = if entry.stage_name.is_empty() { format!("Stage {}", entry.stage + 1) } else { entry.stage_name.clone() };

            game.push(("App", if save.app.is_empty() { "Unknown".to_owned() } else { save.app.clone() }));
            game.push(("Version", if save.version.is_empty() { "Unknown".to_owned() } else { save.version.clone() }));
            game.push(("Map", map));
            stage = Some((named, entry.crown + 1));
        }
        None => game.push(("Save", "Missing".to_owned())),
    }

    Details {
        title,
        meta,
        game,
        stage,
        version: summary.save.as_ref().map(|save| save.version.clone()),
        ready: summary.save.is_some() && summary.frames.is_some(),
    }
}

fn empty_frame(theme: &Theme) -> container::Style {
    let palette = theme.palette();

    container::Style {
        background: Some(Color { a: EMPTY_FILL, ..palette.text }.into()),
        border: Border::default().rounded(6.0).width(1.0).color(Color { a: EDGE_ALPHA, ..palette.text }),
        ..container::Style::default()
    }
}

fn badge_fill(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Color { a: BADGE_FILL, ..Color::BLACK }.into()),
        border: Border::default().rounded(6.0),
        ..container::Style::default()
    }
}

fn corner(content: Element<'_, Message>, across: Horizontal, down: Vertical) -> Element<'_, Message> {
    container(content).width(Length::Fill).height(Length::Fill).align_x(across).align_y(down).into()
}

fn content_rule<'a>() -> Element<'a, Message> {
    container(rule::horizontal(RULE_THICKNESS)).width(Length::Fixed(CONTENT_WIDTH)).into()
}

fn content_card<'a>(title: &'a str, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(section(title, Length::Fill, content))
        .width(Length::Fixed(CONTENT_WIDTH))
        .padding(CARD_PADDING)
        .style(theme::card_container_outlined)
        .into()
}

fn labeled<'a>(label: &'static str, value: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    row![text(label).size(LABEL_SIZE).width(Length::Fixed(FIELD_LABEL_WIDTH)), value.into()].align_y(Vertical::Center).into()
}

fn rows<'a>(entries: &'a [(&'static str, String)]) -> Column<'a, Message> {
    let mut listed = Column::new().spacing(ROW_SPACING);

    for (label, value) in entries {
        let shown: Element<'_, Message> = if *label == "Time" { text_with_superscript(value, LABEL_SIZE) } else { text(value.as_str()).size(LABEL_SIZE).into() };

        listed = listed.push(labeled(label, shown));
    }

    listed
}

impl State {
    pub fn new() -> Self {
        let scale = ((CONTENT_WIDTH - CARD_PADDING * 2.0) / Metrics::FULL.grid_width()).clamp(SMALLEST, 1.0);

        Self { inspector: cat::State::inspector(SCOPE, 0), metrics: Some(Metrics::at(scale)), ..Self::default() }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        if self.open.is_none() {
            return iced::Subscription::none();
        }

        self.inspector.subscription().map(Message::Cat)
    }

    pub fn refresh(&mut self, settings: &Settings) -> Task<Message> {
        self.bundles = tape::list(&tape::library());
        self.manage.refresh(settings, &self.bundles);
        self.filter();

        let picked = match self.picked.take() {
            Some(Pick::Bundle(path)) if !self.bundles.contains(&path) => None,
            other => other,
        };

        self.select(picked.unwrap_or(Pick::Latest), settings.scanner_config(None))
    }

    pub fn relist(&mut self, config: ScannerConfig) -> Task<Message> {
        self.bundles = tape::list(&tape::library());
        self.filter();

        match &self.picked {
            Some(Pick::Bundle(path)) if !self.bundles.contains(path) => self.select(Pick::Latest, config),
            _ => Task::none(),
        }
    }

    fn title(&self) -> String {
        match &self.picked {
            Some(Pick::Bundle(path)) => bundle_name(path),
            _ => LATEST_LABEL.to_owned(),
        }
    }

    fn select(&mut self, pick: Pick, config: ScannerConfig) -> Task<Message> {
        let target = target_of(&pick);
        let stamp = target.as_deref().and_then(tape::stamp);
        let cached = stamp
            .as_ref()
            .and_then(|stamp| self.cache.iter().find(|(held, seen, _)| *held == pick && seen == stamp))
            .map(|(_, _, staged)| Arc::clone(staged));
        let staged = match (cached, target) {
            (Some(staged), _) => Ok(staged),
            (None, Some(target)) => tape::stage(&target, &config).map(Arc::new),
            (None, None) => Err("there is no state folder".to_owned()),
        };

        self.rename = match &pick {
            Pick::Bundle(path) => bundle_name(path),
            Pick::Latest => String::new(),
        };
        self.picked = Some(pick.clone());
        self.open = None;

        match staged {
            Ok(staged) => {
                if let Some(stamp) = stamp {
                    let bundle = matches!(pick, Pick::Bundle(_));

                    self.cache.retain(|(held, _, _)| *held != pick && !(bundle && matches!(held, Pick::Bundle(_))));
                    self.cache.push((pick, stamp, Arc::clone(&staged)));
                }

                self.failure = None;
                self.adopt(staged)
            }
            Err(reason) => {
                warn!("Replay could not be read: {reason}");
                self.details = Some(describe(self.title(), &Summary::default()));
                self.failure = Some(reason);
                self.staged = None;
                self.tiles.clear();
                self.icons.clear();
                self.coin = None;

                Task::none()
            }
        }
    }

    fn adopt(&mut self, staged: Arc<Staged>) -> Task<Message> {
        self.details = Some(describe(self.title(), &staged.summary));

        let adopted = self.inspector.adopt_cats(&staged.cats, &staged.vault).map(Message::Cat);
        let orbs = self.orbs.load(&staged.vault).map(Message::Orbs);

        self.coin = staged.vault.vfs.find(NP_ICON).and_then(|path| image::open(path).ok()).map(|opened| {
            let cropped = kore::common::gfx::autocrop(opened.to_rgba8());

            Handle::from_rgba(cropped.width(), cropped.height(), cropped.into_raw())
        });

        self.decoded.borrow_mut().clear();
        self.icons.clear();

        let tiles: Vec<Tile> = staged
            .summary
            .save
            .iter()
            .flat_map(|save| save.setup.lineup.iter().enumerate().map(move |(slot, unit)| (save, slot, unit)))
            .map(|(save, slot, unit)| self.tile(unit, save.altar_cap, save.costs.get(slot).copied()))
            .collect();

        let deck: Vec<String> = staged.summary.save.as_ref().map(|save| save.icons.clone()).unwrap_or_default();

        for (slot, tile) in tiles.iter().enumerate() {
            let drawn = deck.get(slot).filter(|name| !name.is_empty()).and_then(|name| staged.vault.vfs.find(name.as_str()));
            let scanned = self.inspector.cat(tile.id).and_then(|cat| scanner::deploy_icon(&staged.vault.vfs, cat, tile.form));
            let path = drawn.or(scanned).or_else(|| {
                FORM_LETTERS.get(tile.form).and_then(|letter| staged.vault.vfs.find(format!("uni{:03}_{letter}00.png", tile.id).as_str()))
            });

            if let Some(icon) = path.and_then(|path| header_icon::load(&self.decoded, &path)) {
                self.icons.insert((tile.id, tile.form), icon.handle);
            }
        }

        self.tiles = tiles;
        self.staged = Some(staged);

        Task::batch([adopted, orbs])
    }

    fn tile(&self, unit: &Unit, altar_cap: Option<i32>, cost: Option<i32>) -> Tile {
        let id = u32::try_from(unit.unit).unwrap_or(0);
        let groups = self.inspector.cat(id).and_then(|cat| cat.talent_data.as_ref());
        let talents = unit
            .talents
            .iter()
            .filter_map(|(ability, level)| {
                let index = groups?.groups.iter().position(|group| i32::from(group.ability_id) == *ability)?;

                Some((u8::try_from(index).ok()?, u8::try_from(*level).ok()?))
            })
            .collect();
        let mut worn = unit.orbs.clone();

        worn.sort_unstable();

        Tile {
            id,
            form: usize::try_from(unit.form).unwrap_or(0),
            level: level_label(clamped(unit.level, unit.plus, altar_cap)),
            cost: cost.filter(|cost| *cost >= 0).map(|cost| format!("{cost}\u{00a2}")),
            inspected: match clamped(unit.level, unit.plus, altar_cap) {
                (level, 0) => level.to_string(),
                (level, plus) => format!("{level}+{plus}"),
            },
            talents,
            talented: !unit.talents.is_empty(),
            orbs: worn.into_iter().filter_map(|(_, orb)| u32::try_from(orb).ok()).collect(),
        }
    }

    fn vault(&self) -> Option<&Vault> {
        self.staged.as_deref().map(|staged| &staged.vault)
    }

    pub fn foreign_version(&self) -> Option<&str> {
        let recorded = self.details.as_ref()?.version.as_deref()?;

        (recorded != VERSION).then_some(recorded)
    }

    pub fn target(&self) -> Option<PathBuf> {
        target_of(self.picked.as_ref()?)
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

    pub fn unit_open(&self) -> bool {
        self.open.is_some()
    }

    pub fn update(&mut self, message: Message, settings: &mut Settings, app_state: &mut AppState, ctx: GlobalContext<'_>) -> Task<Message> {
        match message {
            Message::Select(pick) => {
                self.manage.forget_confirm();

                self.select(pick, settings.scanner_config(None))
            }
            Message::Search(query) => {
                self.search = query;
                self.filter();

                Task::none()
            }
            Message::RenameInput(name) => {
                self.rename = tape::clean_name(&name);
                self.rename_live();

                Task::none()
            }
            Message::CommitRename => {
                self.settle_rename();

                Task::none()
            }
            Message::Press(slot) => {
                let now = Instant::now();
                let doubled = self.clicked.take().is_some_and(|(held, at)| held == slot && now.duration_since(at) <= DOUBLE_CLICK);

                if !doubled {
                    self.clicked = Some((slot, now));

                    return Task::none();
                }

                let Some(tile) = self.tiles.get(slot) else {
                    return Task::none();
                };

                self.inspector.inspect(tile.id, tile.form, &tile.inspected, &tile.talents);
                self.open = Some(slot);

                Task::none()
            }
            Message::Cat(msg) => {
                if !msg.views_only() {
                    return Task::none();
                }

                let Some(vault) = self.staged.as_deref().map(|staged| &staged.vault) else {
                    return Task::none();
                };
                let scoped = GlobalContext { param: ctx.param, localizable: ctx.localizable, vault };

                self.inspector.update(msg, settings, app_state, scoped).map(Message::Cat)
            }
            Message::Orbs(msg) => {
                self.orbs.update(msg, None, Allowance::default());

                Task::none()
            }
            Message::UnitPopup(msg) => {
                if self.unit_popup.update(msg, UNIT_POPUP) {
                    self.open = None;
                }

                Task::none()
            }
            Message::Manage(msg) => {
                let title = self.title();
                let (task, effect) = self.manage.update(msg, self.picked.as_ref(), &title, settings, &ctx.vault.vfs);
                let task = task.map(Message::Manage);

                match effect {
                    manage::Effect::None => task,
                    manage::Effect::Saved => Task::batch([task, self.refresh(settings)]),
                    manage::Effect::Deleted(bundle) => {
                        self.cache.retain(|(pick, _, _)| *pick != Pick::Bundle(bundle.clone()));

                        Task::batch([task, self.refresh(settings)])
                    }
                }
            }
        }
    }

    fn filter(&mut self) {
        let query = self.search.to_lowercase();

        self.names = self.bundles.iter().map(|path| bundle_name(path)).collect();
        self.shown = self
            .names
            .iter()
            .enumerate()
            .filter(|(_, name)| query.is_empty() || name.to_lowercase().contains(&query))
            .map(|(index, _)| index)
            .collect();
    }

    fn rename_live(&mut self) {
        let Some(Pick::Bundle(old)) = self.picked.clone() else {
            return;
        };

        if self.manage.busy(&Pick::Bundle(old.clone())) {
            return;
        }

        let renamed = match tape::rename(&old, &self.rename) {
            Ok(Some(renamed)) => renamed,
            Ok(None) => return,
            Err(reason) => {
                warn!("Replay could not be renamed: {reason}");

                return;
            }
        };
        let stamp = tape::stamp(&renamed);
        let stale = Pick::Bundle(old.clone());
        let fresh = Pick::Bundle(renamed.clone());
        let name = bundle_name(&renamed);

        for (pick, seen, _) in &mut self.cache {
            if *pick == stale
                && let Some(stamp) = &stamp
            {
                *pick = fresh.clone();
                seen.clone_from(stamp);
            }
        }

        if let Some(held) = self.bundles.iter_mut().find(|held| **held == old) {
            *held = renamed;
        }

        if let Some(details) = &mut self.details {
            details.title.clone_from(&name);
        }

        self.manage.rekey(&stale, fresh.clone());
        self.picked = Some(fresh);
        self.filter();
    }

    fn settle_rename(&mut self) {
        if let Some(Pick::Bundle(path)) = &self.picked {
            self.rename = bundle_name(path);
        }
    }

    pub fn unit_popup_view<'a>(
        &'a self,
        window: Size,
        settings: &'a Settings,
        app_state: &'a AppState,
        ctx: GlobalContext<'a>,
    ) -> Option<Element<'a, Message>> {
        let tile = self.tiles.get(self.open?)?;
        let vault = self.vault()?;
        let scoped = GlobalContext { param: ctx.param, localizable: ctx.localizable, vault };
        let title = self.inspector.cat(tile.id).map_or("Unit", |cat| cat.names.get(tile.form).and_then(Option::as_deref).unwrap_or("Unit"));

        Some(self.unit_popup.view(
            title,
            UNIT_POPUP,
            window,
            Message::UnitPopup,
            move || self.inspector.detail_view(settings, app_state, scoped, self.orb_header(tile)).map(Message::Cat),
            None,
        ))
    }

    fn orb_header<'a>(&'a self, tile: &'a Tile) -> Option<Element<'a, cat::Message>> {
        if tile.form < FIRST_TALENT_FORM || tile.orbs.is_empty() {
            return None;
        }

        let mut slots = Row::new().spacing(6);

        for orb in &tile.orbs {
            slots = slots.push(container(self.orbs.picture(*orb, SLOT_ORB_SIZE)).padding(2));
        }

        Some(column![theme::bold_text("Orbs").size(ORB_HEADER_SIZE), slots].spacing(6).align_x(Horizontal::Center).into())
    }

    fn view_list(&self) -> Element<'_, Message> {
        let search = text_input("Search Replays...", &self.search)
            .on_input(Message::Search)
            .padding(4)
            .size(13)
            .width(Length::Fill)
            .style(theme::rounded_input);

        let latest_picked = self.picked == Some(Pick::Latest);
        let latest = button(theme::button_label(LATEST_LABEL).size(13))
            .on_press(Message::Select(Pick::Latest))
            .padding([4, 8])
            .width(Length::Fill)
            .style(move |theme: &Theme, status| theme::toggle_button(theme, status, latest_picked));

        let list: Element<'_, Message> = if self.shown.is_empty() {
            container(theme::centered_text("No Replays Found").size(13).style(text::danger))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        } else {
            let mut rows = Column::with_capacity(self.shown.len()).spacing(LIST_SPACING).width(Length::Fill);

            for (path, name) in self.shown.iter().filter_map(|index| Some((self.bundles.get(*index)?, self.names.get(*index)?))) {
                let picked = matches!(&self.picked, Some(Pick::Bundle(held)) if held == path);
                let face = row![
                    Space::new().width(Length::Fixed(EDGE_GAP)),
                    theme::centered_text(name.as_str()).size(ROW_TEXT_SIZE).width(Length::Fill),
                    Space::new().width(Length::Fixed(EDGE_GAP)),
                ]
                    .align_y(Vertical::Center)
                    .height(Length::Fill);

                rows = rows.push(list_row(
                    container(face).width(Length::Fill).height(Length::Fixed(ROW_HEIGHT)),
                    picked,
                    true,
                    Length::Fill,
                    Message::Select(Pick::Bundle(path.clone())),
                ));
            }

            smooth_scroll(scrollable(rows).spacing(SCROLLBAR_GAP).height(Length::Fill).width(Length::Fill)).into()
        };

        container(column![search, Space::new().height(SEARCH_GAP), latest, Space::new().height(LIST_GAP), list].height(Length::Fill))
            .width(Length::Fixed(LIST_WIDTH + LIST_PADDING * 2.0))
            .height(Length::Fill)
            .padding(LIST_PADDING)
            .style(theme::list_panel_container)
            .into()
    }

    fn view_tile(&self, metrics: Metrics, slot: usize, tile: &Tile) -> Element<'_, Message> {
        let (width, height) = (Length::Fixed(metrics.cell_width), Length::Fixed(metrics.cell_height));
        let face: Element<'_, Message> = self.icons.get(&(tile.id, tile.form)).map_or_else(
            || {
                container(text(format!("{:03}-{}", tile.id, tile.form + 1)).size(metrics.text(BADGE_SIZE)))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            },
            |handle| iced_image(handle.clone()).width(width).height(height).into(),
        );
        let badge = container(theme::bold_text(tile.level.as_str()).size(metrics.text(BADGE_SIZE)).color(Color::WHITE))
            .padding([1, 6])
            .style(badge_fill);
        let orb_size = ORB_SIZE * metrics.scale;
        let spread = tile.orbs.len().saturating_sub(1) as f32 * orb_size * ORB_STEP + orb_size;
        let mut worn = stack![Space::new().width(Length::Fixed(spread)).height(Length::Fixed(orb_size))];

        for (place, orb) in tile.orbs.iter().enumerate() {
            worn = worn.push(container(self.orbs.picture(*orb, orb_size)).padding(Padding::default().left(place as f32 * orb_size * ORB_STEP)));
        }

        let coin: Element<'_, Message> = self
            .coin
            .clone()
            .filter(|_| tile.talented)
            .map_or_else(|| Space::new().into(), |handle| iced_image(handle).height(Length::Fixed(COIN_SIZE * metrics.scale)).into());
        let price: Element<'_, Message> = tile.cost.as_deref().map_or_else(
            || Space::new().into(),
            |cost| container(theme::bold_text(cost).size(metrics.text(BADGE_SIZE)).color(Color::WHITE)).padding([1, 6]).style(badge_fill).into(),
        );
        let shown = stack![
            container(face).width(width).height(height).style(empty_frame),
            corner(worn.into(), Horizontal::Left, Vertical::Top),
            corner(coin, Horizontal::Left, Vertical::Bottom),
            corner(badge.into(), Horizontal::Right, Vertical::Top),
            corner(price, Horizontal::Right, Vertical::Bottom),
        ]
            .width(width)
            .height(height);

        mouse_area(shown).interaction(mouse::Interaction::Pointer).on_press(Message::Press(slot)).into()
    }

    fn view_lineup(&self) -> Element<'_, Message> {
        let metrics = self.metrics.unwrap_or(Metrics::FULL);
        let mut grid = Column::new().spacing(metrics.gap);

        for line in 0..LINES {
            let mut cells = Row::new().spacing(metrics.gap);

            for column in 0..COLUMNS {
                let slot = line * COLUMNS + column;
                let cell: Element<'_, Message> = match self.tiles.get(slot) {
                    Some(tile) => self.view_tile(metrics, slot, tile),
                    None => container(Space::new())
                        .width(Length::Fixed(metrics.cell_width))
                        .height(Length::Fixed(metrics.cell_height))
                        .style(empty_frame)
                        .into(),
                };

                cells = cells.push(cell);
            }

            grid = grid.push(cells);
        }

        grid.into()
    }

    fn view_details(&self) -> Element<'_, Message> {
        let Some(details) = &self.details else {
            return Space::new().into();
        };

        let mut game = rows(&details.game);

        if let Some((named, crowns)) = &details.stage {
            let stage = row![
                text(format!("{named} ({crowns}")).size(LABEL_SIZE),
                text(CROWN_GLYPH).font(fonts::MISC_SYMBOLS).size(LABEL_SIZE),
                text(")").size(LABEL_SIZE),
            ]
                .align_y(Vertical::Center);

            game = game.push(labeled("Stage", stage));
        }


        let title: Element<'_, Message> = match &self.picked {
            Some(Pick::Bundle(_)) => text_input("Replay Title", &self.rename)
                .on_input(Message::RenameInput)
                .on_submit(Message::CommitRename)
                .size(TITLE_SIZE)
                .padding(TITLE_PADDING)
                .width(Length::Fixed(CONTENT_WIDTH))
                .style(theme::rounded_input)
                .into(),
            _ => text_input("", details.title.as_str())
                .size(TITLE_SIZE)
                .padding(TITLE_PADDING)
                .width(Length::Fixed(CONTENT_WIDTH))
                .style(|theme: &Theme, _| theme::rounded_input(theme, text_input::Status::Active))
                .into(),
        };

        let mut content = column![
            title,
            Space::new().height(RULE_PADDING),
            content_rule(),
            Space::new().height(RULE_PADDING),
            content_card("Meta", rows(&details.meta)),
            Space::new().height(SECTION_GAP),
            content_card("Game", game),
            Space::new().height(SECTION_GAP),
            content_card("Lineup", self.view_lineup()),
            Space::new().height(SECTION_GAP),
            content_card("Manage", self.manage.view(self.picked.as_ref(), details.ready).map(Message::Manage)),
        ];

        if let Some(reason) = &self.failure {
            content = content.push(Space::new().height(SECTION_GAP));
            content = content.push(text(format!("The replay's files could not be read: {reason}")).size(LABEL_SIZE));
        }

        content.into()
    }

    pub fn view(&self, bottom: f32) -> Element<'_, Message> {
        let page = container(self.view_details()).padding(Padding { bottom, ..Padding::new(DETAILS_PADDING) }).width(Length::Fill);

        row![self.view_list(), smooth_scroll(scrollable(page).width(Length::Fill).height(Length::Fill))]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
