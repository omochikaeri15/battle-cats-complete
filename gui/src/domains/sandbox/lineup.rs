use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::alignment::{Horizontal, Vertical};
use iced::widget::image::Handle;
use iced::widget::{button, column, container, image as iced_image, mouse_area, row, scrollable, sensor, stack, text, text_input, Column, Row, Space};
use iced::{mouse, Border, Color, Element, Length, Padding, Point, Size, Task, Theme};

use kore::common::context::GlobalContext;
use kore::domains::cat::scanner::{self, CatEntry};
use kore::domains::cat::game::stats::get_final_stats;
use kore::domains::sandbox::orb::Allowance;
use kore::domains::sandbox::rules::Rules;
use kore::domains::sandbox::{mount_of, Cell, Lineup, Member, Roster, BENCH_SLOTS, LINEUP_SLOTS};
use kore::domains::settings::Settings;
use kore::Vfs;
use kore::Source;
use kore::common::gfx::open_image;
use nyanko::combat::Entity;

use crate::app::state::AppState;
use crate::app::theme;
use crate::common::feedback::{Slot as Confirm, CONFIRM_SHORT_LABEL};
use crate::common::header_icon;
use crate::domains::cat::{self, Picked};
use crate::editor;
use crate::widget::{list_row, popup, roster_list, smooth_scroll};

use super::{combos, orbs};

pub(super) const LIST_SCOPE: &str = "sandbox-cat-banner-list";

const UNIT_POPUP: popup::Spec = popup::Spec::new(popup::Kind::SandboxUnit, Size::new(760.0, 540.0));
const ORB_POPUP: popup::Spec = popup::Spec::new(popup::Kind::SandboxOrb, Size::new(310.0, 250.0));
const RIPE: Duration = Duration::from_millis(160);
const DOUBLE_CLICK: Duration = Duration::from_millis(400);
const SLACK: f32 = 6.0;
const FIRST_TALENT_FORM: usize = 2;

const LEFT_WIDTH: f32 = roster_list::LIST_WIDTH + 16.0;
const RIGHT_WIDTH: f32 = 220.0;
const TOP_PAD: f32 = 10.0;
const CARD_GAP: f32 = 10.0;
const COLUMNS: usize = 5;
const DECK_SHARE: f32 = 0.68;
pub(super) const SMALLEST: f32 = 0.45;
const TEXT_FLOOR: f32 = 0.8;

const LEVEL_SIZE: f32 = 11.0;
const BADGE_SIZE: f32 = 12.0;
const COST_SIZE: f32 = 12.0;
const BARRED_SHADE: f32 = 0.6;
const ORB_SIZE: f32 = 26.0;
const COIN_SIZE: f32 = 18.0;
const ORB_STEP: f32 = 0.55;
const NP_ICON: &str = "gatyaitemD_07_f.png";
const SLOT_ORB_SIZE: f32 = 44.0;
const BADGE_FILL: f32 = 0.62;
const GHOST_ALPHA: f32 = 0.75;
const EMPTY_FILL: f32 = 0.07;
const NAME_SIZE: f32 = 13.0;
const ORB_HEADER_SIZE: f32 = 18.0;
const ECHO: Duration = Duration::from_millis(80);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Metrics {
    pub(super) scale: f32,
    pad: f32,
    pub(super) cell_width: f32,
    pub(super) cell_height: f32,
    pub(super) gap: f32,
    header: f32,
    header_gap: f32,
    bench_gap: f32,
    bench_title: f32,
    bench_note: f32,
}

impl Metrics {
    pub(super) const FULL: Self = Self::at(1.0);

    pub(super) const fn at(scale: f32) -> Self {
        Self {
            scale,
            pad: 12.0 * scale,
            cell_width: 110.0 * scale,
            cell_height: 85.0 * scale,
            gap: 6.0 * scale,
            header: 22.0 * scale,
            header_gap: 8.0 * scale,
            bench_gap: 10.0 * scale,
            bench_title: 18.0 * scale,
            bench_note: 14.0 * scale,
        }
    }

    fn fitting(area: Size) -> Self {
        if area.width <= 0.0 || area.height <= 0.0 {
            return Self::FULL;
        }

        let across = (area.width - LEFT_WIDTH - RIGHT_WIDTH - CARD_GAP * 2.0) / Self::FULL.deck_width();
        let down = area.height * DECK_SHARE / Self::FULL.deck_height();

        Self::at(across.min(down).clamp(SMALLEST, 1.0))
    }

    pub(super) fn grid_width(&self) -> f32 {
        self.cell_width * COLUMNS as f32 + self.gap * (COLUMNS as f32 - 1.0)
    }

    fn deck_width(&self) -> f32 {
        self.grid_width() + self.pad * 2.0
    }

    fn grid_top(&self) -> f32 {
        self.pad + self.header + self.header_gap
    }

    fn bench_top(&self) -> f32 {
        self.grid_top() + self.cell_height * 2.0 + self.gap + self.bench_gap + self.bench_title + self.bench_note + self.gap
    }

    fn deck_height(&self) -> f32 {
        self.bench_top() + self.cell_height + self.pad
    }

    pub(super) fn text(&self, size: f32) -> f32 {
        size * self.scale.max(TEXT_FLOOR)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Cargo {
    Fresh(u32),
    Held(Cell),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Drag {
    #[default]
    Idle,
    Pressed {
        cargo: Cargo,
        since: Instant,
        from: Option<Point>,
    },
    Moving {
        cargo: Cargo,
        at: Point,
        onto: Option<Cell>,
    },
}

#[derive(Clone, Debug)]
pub enum Message {
    Cat(cat::Message),
    Combos(combos::Message),
    Orbs(orbs::Message),
    Measured(Size),
    PressCell(Cell),
    DragMove(Point),
    DragEnd,
    DragCancel,
    UnitPopup(popup::Message),
    OrbPopup(popup::Message),
    Drop(Cell),
    Create,
    Delete,
    DeleteExpired,
    Load(usize),
    Rename(String),
    Search(String),
}

fn abandoned(event: iced::Event, _status: iced::event::Status, _window: iced::window::Id) -> Option<Message> {
    matches!(event, iced::Event::Mouse(mouse::Event::CursorLeft) | iced::Event::Window(iced::window::Event::Unfocused)).then_some(Message::DragCancel)
}

#[derive(Clone, Copy, Debug, Default)]
struct Priced {
    cost: i32,
    barred: bool,
    talented: bool,
}

pub struct State {
    inspector: cat::State,
    combos: combos::State,
    orbs: orbs::State,
    drag: Drag,
    area: Size,
    metrics: Metrics,
    rules: Rules,
    priced: HashMap<Cell, Priced>,
    dropped_at: Option<Instant>,
    clicked: Option<(Cell, Instant)>,
    icons: HashMap<(u32, usize), Handle>,
    decoded: header_icon::Cache,
    coin: Option<Handle>,
    banner_form: usize,
    editing: Option<u32>,
    unit_popup: popup::State,
    orb_popup: popup::State,
    orb_slot: Option<usize>,
    allowance: Allowance,
    deleting: Confirm<()>,
    search: String,
    found: Vec<usize>,
    altar: Option<u32>,
    orphans: HashMap<u32, [Option<Entity>; 4]>,
    mount: String,
    orb_slots: Arc<HashMap<u32, usize>>,
}

impl State {
    pub fn new(banner_form: usize) -> Self {
        Self {
            inspector: cat::State::inspector(LIST_SCOPE, banner_form, popup::Kind::SandboxAnimationExport),
            combos: combos::State::default(),
            orbs: orbs::State::default(),
            drag: Drag::Idle,
            area: Size::ZERO,
            metrics: Metrics::FULL,
            rules: Rules::default(),
            priced: HashMap::new(),
            dropped_at: None,
            clicked: None,
            icons: HashMap::new(),
            decoded: header_icon::Cache::default(),
            coin: None,
            banner_form,
            editing: None,
            unit_popup: popup::State::default(),
            orb_popup: popup::State::default(),
            orb_slot: None,
            allowance: Allowance::default(),
            deleting: Confirm::default(),
            search: String::new(),
            found: Vec::new(),
            altar: None,
            orphans: HashMap::new(),
            mount: String::new(),
            orb_slots: Arc::default(),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        let inspecting = if self.editing.is_some() { self.inspector.subscription().map(Message::Cat) } else { iced::Subscription::none() };
        let dragging = if self.drag == Drag::Idle { iced::Subscription::none() } else { iced::event::listen_with(abandoned) };

        iced::Subscription::batch([inspecting, dragging])
    }

    pub fn icon_stream(&mut self) -> Task<Message> {
        self.inspector.icon_stream().map(Message::Cat)
    }

    pub fn adopt_cats(&mut self, cats: &[CatEntry], app_state: &AppState, ctx: GlobalContext<'_>) -> Task<Message> {
        let adopted = self.inspector.adopt_cats(cats, ctx.vault).map(Message::Cat);
        let orbs = self.orbs.load(ctx.vault).map(Message::Orbs);

        self.combos.invalidate();
        self.decoded.borrow_mut().clear();
        self.coin = None;
        self.settle(app_state, ctx);

        Task::batch([adopted, orbs])
    }

    pub fn set_banner_form(&mut self, banner_form: usize) {
        self.banner_form = banner_form;
        self.inspector.set_banner_form(banner_form);
    }

    pub fn enter(&mut self, app_state: &AppState, ctx: GlobalContext<'_>) -> Task<Message> {
        self.settle(app_state, ctx);

        self.inspector.restore_list(app_state.sandbox.list_scroll_offset, &app_state.sandbox.search_query).map(Message::Cat)
    }

    pub fn unit_open(&self) -> bool {
        self.editing.is_some()
    }

    pub fn orb_open(&self) -> bool {
        self.editing.is_some() && self.orb_slot.is_some()
    }

    pub fn export_open(&self) -> bool {
        self.editing.is_some() && self.inspector.export_popup_open()
    }

    pub fn export_popup_view(&self, window: Size) -> Option<Element<'_, Message>> {
        (self.editing.is_some() && self.inspector.export_popup_visible())
            .then(|| self.inspector.export_popup_view(window))
            .flatten()
            .map(|view| view.map(Message::Cat))
    }

    pub fn filter_open(&self) -> bool {
        self.inspector.filter_popup_open()
    }

    pub fn inspector(&self) -> &cat::State {
        &self.inspector
    }

    pub fn worn<'a>(&'a self, member: &'a Member) -> impl Iterator<Item = (usize, u32)> + 'a {
        member.worn(self.orb_slots.get(&member.id).copied().unwrap_or(0), self.orbs.catalog())
    }

    pub fn set_altar(&mut self, cap: Option<u32>) {
        self.altar = cap;
    }

    pub fn set_rules(&mut self, rules: Rules, app_state: &AppState) {
        if self.rules == rules {
            return;
        }

        self.rules = rules;
        self.combos.ban(&self.rules);
        self.reprice(app_state);
    }

    fn reprice(&mut self, app_state: &AppState) {
        self.priced.clear();

        let Some(lineup) = app_state.sandbox.roster.current() else {
            return;
        };

        let slots = lineup.slots.iter().enumerate().map(|(index, member)| (Cell::Slot(index), member));
        let bench = lineup.bench.iter().enumerate().filter_map(|(index, member)| Some((Cell::Bench(index), member.as_ref()?)));
        let mut fielded: Vec<usize> = Vec::new();

        for (cell, member) in slots.chain(bench) {
            let cat = self.inspector.cat(member.id);
            let stats = match cat {
                Some(cat) => cat.stats.get(member.form).and_then(Option::as_ref),
                None => self.orphans.get(&member.id).and_then(|stats| stats.get(member.form)).and_then(Option::as_ref),
            };
            let talents = (member.form >= FIRST_TALENT_FORM).then_some(&member.talents);
            let (level, plus) = member.levels();
            let base = stats.map_or(0, |stats| {
                get_final_stats(stats, cat.and_then(|cat| cat.curve.as_ref()), (level + plus) as i32, cat.and_then(|cat| cat.talent_data.as_ref()), talents)
                    .eoc1_cost
            });
            let rarity = cat.and_then(|cat| usize::try_from(cat.unitbuy.rarity).ok()).unwrap_or(0);
            let cost = self.rules.cost(base, rarity);
            let groups = cat.and_then(|cat| cat.talent_data.as_ref()).map_or(0, |talent| talent.groups.len());
            let talented = member.talented() && member.talents.iter().any(|(index, level)| *level > 0 && usize::from(*index) < groups);
            let earlier = fielded.iter().filter(|held| **held == rarity).count();

            self.priced.insert(cell, Priced { cost, talented, barred: self.rules.bars(member.id, rarity, cost, earlier, match cell {
                Cell::Slot(slot) => Some(slot),
                Cell::Bench(_) => None,
            }) });

            if matches!(cell, Cell::Slot(_)) {
                fielded.push(rarity);
            }
        }
    }

    fn settle(&mut self, app_state: &AppState, ctx: GlobalContext<'_>) {
        self.orphans.clear();
        self.mount = mount_of(&ctx.vault.vfs);
        self.orb_slots = ctx.vault.vds.cats.orb_slots(&ctx.vault.vfs);

        let members = app_state.sandbox.roster.current().into_iter().flat_map(|lineup| lineup.slots.iter().chain(lineup.bench.iter().flatten()));

        for member in members {
            if self.inspector.cat(member.id).is_none() && !self.orphans.contains_key(&member.id) {
                self.orphans.insert(member.id, scanner::orphan_stats(&ctx.vault.vfs, member.id));
            }
        }

        self.reprice(app_state);

        let roster = &app_state.sandbox.roster;
        let lineup = roster.current();

        self.icons.clear();

        if self.coin.is_none() {
            self.coin = ctx.vault.vfs.find(NP_ICON).and_then(|path| open_image(&ctx.vault.vfs.source(&path))).map(|opened| {
                let cropped = kore::common::gfx::autocrop(opened.to_rgba8());

                Handle::from_rgba(cropped.width(), cropped.height(), cropped.into_raw())
            });
        }

        for member in lineup.into_iter().flat_map(|lineup| lineup.slots.iter().chain(lineup.bench.iter().flatten())) {
            let path = match self.inspector.cat(member.id) {
                Some(cat) => scanner::deploy_icon(&ctx.vault.vfs, cat, member.form),
                None => scanner::orphan_icon(&ctx.vault.vfs, member.id, member.form),
            };

            if let Some(icon) = path.and_then(|path| header_icon::load(&self.decoded, &ctx.vault.vfs.source(&path))) {
                self.icons.insert((member.id, member.form), icon.handle);
            }
        }

        let wanted = self.search.to_lowercase();

        self.found = roster
            .lineups
            .iter()
            .enumerate()
            .filter(|(_, lineup)| wanted.is_empty() || lineup.name.to_lowercase().contains(&wanted))
            .map(|(index, _)| index)
            .collect();

        self.combos.ensure(ctx, lineup);
        self.combos.refresh(ctx, lineup);
    }

    fn recruit(&self, id: u32, form: Option<usize>, settings: &Settings, roster: &Roster) -> Option<Member> {
        let (lowest, level) = self.inspector.seeded(id, self.banner_form, settings)?;
        let slots = 0;

        let fresh = Member { id, form: form.unwrap_or(lowest), level, talents: HashMap::new(), orbs: vec![None; slots] };

        Some(roster.dress(fresh, form, &self.mount))
    }

    fn open(&mut self, cell: Cell, lineup: &Lineup, vfs: &Vfs) {
        let Some(member) = lineup.get(cell) else {
            return;
        };

        self.inspector.reveal(member.id, member.form, vfs);
        self.inspector.set_export_scope(format!("lineup-{:016x}", lineup.id));
        self.inspector.inspect(member.id, member.form, &member.level, &member.talents);
        self.editing = Some(member.id);
        self.orb_slot = None;
        self.combos.focus(Some((member.id, member.form)));
    }

    fn close(&mut self) {
        self.editing = None;
        self.orb_slot = None;
        self.combos.focus(None);
    }

    fn landing(&self, at: Point) -> Option<Cell> {
        if self.area.width <= 0.0 {
            return None;
        }

        let metrics = self.metrics;
        let left = LEFT_WIDTH + ((self.area.width - LEFT_WIDTH - RIGHT_WIDTH - metrics.deck_width()) / 2.0).max(0.0) + metrics.pad;
        let across = at.x - left;

        if !(0.0..metrics.grid_width()).contains(&across) {
            return None;
        }

        let pitch = metrics.cell_width + metrics.gap;
        let column = (across / pitch) as usize;

        if column >= COLUMNS || across - column as f32 * pitch > metrics.cell_width {
            return None;
        }

        let down = at.y - TOP_PAD;
        let grid = down - metrics.grid_top();
        let bench = down - metrics.bench_top();

        if (0.0..metrics.cell_height).contains(&bench) {
            return Some(Cell::Bench(column));
        }

        if !(0.0..metrics.cell_height * 2.0 + metrics.gap).contains(&grid) {
            return None;
        }

        let fall = metrics.cell_height + metrics.gap;
        let line = (grid / fall) as usize;

        (grid - line as f32 * fall <= metrics.cell_height).then_some(Cell::Slot(line * COLUMNS + column))
    }

    fn held(&self, lineup: Option<&Lineup>) -> Option<Cell> {
        lineup?.find(self.editing?)
    }

    pub fn update(&mut self, message: Message, settings: &mut Settings, app_state: &mut AppState, ctx: GlobalContext<'_>) -> Task<Message> {
        let task = self.update_inner(message, settings, app_state, ctx);

        self.settle(app_state, ctx);
        app_state.sandbox.roster.remember(&self.mount);

        task
    }

    fn update_inner(&mut self, message: Message, settings: &mut Settings, app_state: &mut AppState, ctx: GlobalContext<'_>) -> Task<Message> {
        match message {
            Message::Cat(msg) => {
                match msg.picked() {
                    Some(Picked::Pressed(id)) => {
                        self.drag = Drag::Pressed { cargo: Cargo::Fresh(id), since: Instant::now(), from: None };

                        return Task::none();
                    }
                    Some(Picked::Chosen(id)) => {
                        if let Some(member) = self.recruit(id, None, settings, &app_state.sandbox.roster) {
                            app_state.sandbox.roster.current_mut().add(self.fitted(member, ctx));
                        }

                        return Task::none();
                    }
                    None => (),
                }

                if let Some(slot) = cat::State::header_action(&msg) {
                    self.orb_slot = Some(slot);
                    self.allowance = self.allowance(app_state);

                    return Task::none();
                }

                let task = self.inspector.update(msg, settings, app_state, ctx).map(Message::Cat);

                self.write_back(app_state, ctx);
                app_state.sandbox.list_scroll_offset = self.inspector.list_scroll_offset();

                if app_state.sandbox.search_query != self.inspector.search_query {
                    app_state.sandbox.search_query.clone_from(&self.inspector.search_query);
                }

                task
            }
            Message::Combos(msg) => {
                let Some(line) = self.combos.update(msg) else {
                    return Task::none();
                };

                let rows = ctx.vault.vds.cats.combos(&ctx.vault.vfs);

                if let Some(row) = rows.get(line) {
                    let mut lineup = std::mem::take(app_state.sandbox.roster.current_mut());

                    lineup.adopt(row, |id, form| {
                        self.recruit(id, Some(form), settings, &app_state.sandbox.roster)
                            .map_or_else(|| Member { id, form, ..Member::default() }, |member| self.fitted(member, ctx))
                    });

                    *app_state.sandbox.roster.current_mut() = lineup;
                }

                Task::none()
            }
            Message::Orbs(msg) => {
                let cell = self.held(app_state.sandbox.roster.current());
                let allowance = self.allowance;
                let held = self.orb_slot.and_then(|slot| {
                    app_state.sandbox.roster.current()?.get(cell?)?.orbs.get(slot).copied().flatten()
                });

                if let orbs::Outcome::Equip(orb) = self.orbs.update(msg, held, allowance)
                    && let (Some(cell), Some(slot)) = (cell, self.orb_slot)
                    && let Some(member) = app_state.sandbox.roster.current_mut().get_mut(cell)
                    && let Some(target) = member.orbs.get_mut(slot)
                {
                    *target = orb;
                }

                Task::none()
            }
            Message::Measured(size) => {
                self.area = size;
                self.metrics = Metrics::fitting(size);

                Task::none()
            }
            Message::PressCell(cell) => {
                if self.dropped_at.take().is_some_and(|at| at.elapsed() <= ECHO) {
                    return Task::none();
                }

                let filled = app_state.sandbox.roster.current().is_some_and(|lineup| lineup.get(cell).is_some());

                if filled {
                    self.drag = Drag::Pressed { cargo: Cargo::Held(cell), since: Instant::now(), from: None };
                }

                Task::none()
            }
            Message::DragMove(at) => {
                self.drag = match self.drag {
                    Drag::Pressed { cargo, since, from } => {
                        let origin = from.unwrap_or(at);
                        let strayed = (at.x - origin.x).abs() > SLACK || (at.y - origin.y).abs() > SLACK;

                        if strayed || since.elapsed() >= RIPE {
                            Drag::Moving { cargo, at, onto: self.landing(at) }
                        } else {
                            Drag::Pressed { cargo, since, from: Some(origin) }
                        }
                    }
                    Drag::Moving { cargo, .. } => Drag::Moving { cargo, at, onto: self.landing(at) },
                    Drag::Idle => Drag::Idle,
                };

                Task::none()
            }
            Message::DragCancel => {
                self.drag = Drag::Idle;

                Task::none()
            }
            Message::DragEnd => {
                let settled = std::mem::take(&mut self.drag);

                match settled {
                    Drag::Pressed { cargo: Cargo::Fresh(id), .. } => {
                        if let Some(member) = self.recruit(id, None, settings, &app_state.sandbox.roster) {
                            app_state.sandbox.roster.current_mut().add(self.fitted(member, ctx));
                        }
                    }
                    Drag::Pressed { cargo: Cargo::Held(cell), .. } => {
                        let now = Instant::now();
                        let doubled = self.clicked.is_some_and(|(held, at)| held == cell && now.duration_since(at) <= DOUBLE_CLICK);

                        self.clicked = (!doubled).then_some((cell, now));

                        if doubled && let Some(lineup) = app_state.sandbox.roster.current() {
                            self.open(cell, lineup, &ctx.vault.vfs);
                        }
                    }
                    Drag::Moving { cargo: Cargo::Fresh(id), onto: Some(onto), .. } => {
                        if let Some(member) = self.recruit(id, None, settings, &app_state.sandbox.roster) {
                            app_state.sandbox.roster.current_mut().place(self.fitted(member, ctx), onto);
                        }
                    }
                    Drag::Moving { cargo: Cargo::Held(from), onto: Some(onto), .. } => {
                        app_state.sandbox.roster.current_mut().shift(from, onto);
                    }
                    Drag::Moving { cargo: Cargo::Held(from), onto: None, .. } => {
                        app_state.sandbox.roster.current_mut().take(from);
                    }
                    _ => (),
                }

                Task::none()
            }
            Message::UnitPopup(msg) => {
                if self.unit_popup.update(msg, UNIT_POPUP) {
                    self.close();
                }

                Task::none()
            }
            Message::OrbPopup(msg) => {
                if self.orb_popup.update(msg, ORB_POPUP) {
                    self.orb_slot = None;
                }

                Task::none()
            }
            Message::Drop(cell) => {
                let gone = app_state.sandbox.roster.current_mut().take(cell);

                if gone.is_some_and(|member| self.editing == Some(member.id)) {
                    self.close();
                }

                Task::none()
            }
            Message::Create => {
                app_state.sandbox.roster.create();
                self.close();

                Task::none()
            }
            Message::Delete => {
                if !self.deleting.is_set() {
                    return self.deleting.set((), Message::DeleteExpired);
                }

                self.deleting.clear();
                app_state.sandbox.roster.delete();
                self.close();

                Task::none()
            }
            Message::DeleteExpired => {
                self.deleting.expire();

                Task::none()
            }
            Message::Load(index) => {
                if index < app_state.sandbox.roster.lineups.len() {
                    app_state.sandbox.roster.selected = index;
                }

                self.deleting.clear();
                self.close();

                Task::none()
            }
            Message::Rename(name) => {
                app_state.sandbox.roster.current_mut().name = name;

                Task::none()
            }
            Message::Search(query) => {
                self.search = query;

                Task::none()
            }
        }
    }

    fn fitted(&self, mut member: Member, ctx: GlobalContext<'_>) -> Member {
        let slots = ctx.vault.vds.cats.orb_slots(&ctx.vault.vfs).get(&member.id).copied().unwrap_or(0);

        member.orbs.resize(slots, None);
        member
    }

    fn write_back(&mut self, app_state: &mut AppState, ctx: GlobalContext<'_>) {
        let Some(cell) = self.held(app_state.sandbox.roster.current()) else {
            return;
        };
        let Some(seen) = self.inspector.inspected() else {
            return;
        };
        let slots = ctx.vault.vds.cats.orb_slots(&ctx.vault.vfs);

        let Some(member) = app_state.sandbox.roster.current_mut().get_mut(cell) else {
            return;
        };

        member.form = seen.form;
        self.combos.focus(Some((member.id, member.form)));

        if member.level != seen.level {
            member.level = seen.level.to_owned();
        }

        let talents = seen.talents.cloned().unwrap_or_default();

        if member.talents != talents {
            member.talents = talents;
        }

        member.orbs.resize(slots.get(&member.id).copied().unwrap_or(0), None);
    }

    pub fn unit_popup_view<'a>(
        &'a self,
        window: Size,
        settings: &'a Settings,
        app_state: &'a AppState,
        ctx: GlobalContext<'a>,
    ) -> Option<Element<'a, Message>> {
        let lineup = app_state.sandbox.roster.current()?;
        let member = lineup.get(self.held(Some(lineup))?)?;
        let title = self.inspector.cat(member.id).map_or("Unit", |cat| cat.names.get(member.form).and_then(Option::as_deref).unwrap_or("Unit"));

        Some(self.unit_popup.view(
            title,
            UNIT_POPUP,
            window,
            Message::UnitPopup,
            move || self.inspector.detail_view(settings, app_state, ctx, self.orb_header(member)).map(Message::Cat),
            None,
        ))
    }

    fn orb_header<'a>(&'a self, member: &'a Member) -> Option<Element<'a, cat::Message>> {
        if member.form < FIRST_TALENT_FORM || member.orbs.is_empty() {
            return None;
        }

        let mut slots = Row::new().spacing(6);

        for (slot, orb) in member.orbs.iter().enumerate() {
            let face: Element<'a, cat::Message> = orb.map_or_else(
                || container(text("+").size(20)).center_x(Length::Fixed(SLOT_ORB_SIZE)).center_y(Length::Fixed(SLOT_ORB_SIZE)).into(),
                |orb| self.orbs.picture(orb, SLOT_ORB_SIZE),
            );

            slots = slots.push(button(face).padding(2).style(theme::neutral_button).on_press(cat::Message::HeaderAction(slot)));
        }

        Some(column![theme::bold_text("Orbs").size(ORB_HEADER_SIZE), slots].spacing(6).align_x(Horizontal::Center).into())
    }

    fn allowance(&self, app_state: &AppState) -> Allowance {
        let lineup = app_state.sandbox.roster.current();

        self.held(lineup)
            .and_then(|cell| lineup?.get(cell))
            .and_then(|member| self.inspector.cat(member.id))
            .map_or_else(Allowance::default, Allowance::of)
    }

    pub fn orb_popup_view<'a>(&'a self, window: Size, app_state: &'a AppState) -> Option<Element<'a, Message>> {
        let lineup = app_state.sandbox.roster.current()?;
        let slot = self.orb_slot?;
        let held = lineup.get(self.held(Some(lineup))?)?.orbs.get(slot).copied()?;
        let allowance = self.allowance;

        Some(self.orb_popup.view("Orb", ORB_POPUP, window, Message::OrbPopup, move || self.orbs.view(held, allowance).map(Message::Orbs), None))
    }

    pub fn filter_popup_view(&self, window: Size) -> Option<Element<'_, Message>> {
        self.inspector.filter_popup_view(window).map(|view| view.map(Message::Cat))
    }

    pub fn view<'a>(&'a self, app_state: &'a AppState) -> Element<'a, Message> {
        let lineup = app_state.sandbox.roster.current();
        let onto = match self.drag {
            Drag::Moving { onto, .. } => onto,
            _ => None,
        };

        let center = container(
            column![
                self.view_deck(lineup, onto),
                self.combos.view((self.area.width - LEFT_WIDTH - RIGHT_WIDTH - CARD_GAP * 2.0).max(0.0)).map(Message::Combos),
            ]
                .spacing(CARD_GAP)
                .align_x(Horizontal::Center),
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding { top: TOP_PAD, right: CARD_GAP, bottom: CARD_GAP, left: CARD_GAP })
            .align_x(Horizontal::Center);

        let content = row![self.inspector.sidebar_view().map(Message::Cat), center, self.view_saved(app_state)]
            .width(Length::Fill)
            .height(Length::Fill);

        let mut layers = stack![sensor(content).on_show(Message::Measured).on_resize(Message::Measured)];

        if self.drag != Drag::Idle {
            let ghost: Element<'_, Message> = match self.drag {
                Drag::Moving { cargo, at, .. } => self.ghost(cargo, at, lineup, &app_state.sandbox.roster),
                _ => Space::new().width(Length::Fill).height(Length::Fill).into(),
            };

            layers = layers.push(
                mouse_area(ghost)
                    .interaction(mouse::Interaction::Grabbing)
                    .on_move(Message::DragMove)
                    .on_release(Message::DragEnd),
            );
        }

        layers.into()
    }

    fn ghost<'a>(&'a self, cargo: Cargo, at: Point, lineup: Option<&'a Lineup>, roster: &Roster) -> Element<'a, Message> {
        let metrics = self.metrics;
        let handle = match cargo {
            Cargo::Held(cell) => lineup.and_then(|lineup| lineup.get(cell)).and_then(|member| self.icons.get(&(member.id, member.form))).cloned(),
            Cargo::Fresh(id) => self.inspector.cat(id).and_then(|cat| {
                let shown = cat::State::shown_form(cat, self.banner_form);
                let form = roster.recall(id, &self.mount).map_or(shown, |past| past.form);

                cat.deploy_icon_paths
                    .get(form)
                    .or_else(|| cat.deploy_icon_paths.get(shown))
                    .and_then(Option::as_ref)
                    .and_then(|path| header_icon::load(&self.decoded, &Source::disk(path)))
                    .map(|icon| icon.handle)
            }),
        };

        let carried: Element<'a, Message> = handle.map_or_else(
            || Space::new().width(Length::Fixed(metrics.cell_width)).height(Length::Fixed(metrics.cell_height)).into(),
            |handle| {
                iced_image(handle)
                    .width(Length::Fixed(metrics.cell_width))
                    .height(Length::Fixed(metrics.cell_height))
                    .opacity(GHOST_ALPHA)
                    .into()
            },
        );

        let placed = Padding::default()
            .top((at.y - metrics.cell_height / 2.0).max(0.0))
            .left((at.x - metrics.cell_width / 2.0).max(0.0));

        container(carried).padding(placed).width(Length::Fill).height(Length::Fill).into()
    }

    fn view_deck<'a>(&'a self, lineup: Option<&'a Lineup>, onto: Option<Cell>) -> Element<'a, Message> {
        let metrics = self.metrics;
        let mut grid = Column::new().spacing(metrics.gap);

        for line in 0..LINEUP_SLOTS / COLUMNS {
            let mut cells = Row::new().spacing(metrics.gap);

            for column in 0..COLUMNS {
                let cell = Cell::Slot(line * COLUMNS + column);

                cells = cells.push(self.view_cell(cell, lineup.and_then(|lineup| lineup.get(cell)), onto == Some(cell)));
            }

            grid = grid.push(cells);
        }

        let mut bench = Row::new().spacing(metrics.gap);

        for index in 0..BENCH_SLOTS {
            let cell = Cell::Bench(index);

            bench = bench.push(self.view_cell(cell, lineup.and_then(|lineup| lineup.get(cell)), onto == Some(cell)));
        }

        let titled = |label: &'a str, height: f32, size: f32| {
            container(theme::bold_text(label).size(size)).width(Length::Fill).height(Length::Fixed(height)).align_x(Horizontal::Center).align_y(Vertical::Center)
        };

        let note = container(
            text("Does not affect battle")
                .size(metrics.text(LEVEL_SIZE))
                .style(|theme: &Theme| text::Style { color: Some(theme::weak_text_color(theme)) }),
        )
            .width(Length::Fill)
            .height(Length::Fixed(metrics.bench_note))
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center);

        let card = column![
            titled("Lineup", metrics.header, metrics.text(16.0)),
            Space::new().height(Length::Fixed(metrics.header_gap)),
            grid,
            Space::new().height(Length::Fixed(metrics.bench_gap)),
            titled("Bench", metrics.bench_title, metrics.text(14.0)),
            note,
            Space::new().height(Length::Fixed(metrics.gap)),
            bench,
        ];

        container(card)
            .padding(metrics.pad)
            .width(Length::Fixed(metrics.deck_width()))
            .height(Length::Fixed(metrics.deck_height()))
            .style(theme::outlined_card_container)
            .into()
    }

    fn view_cell<'a>(&'a self, cell: Cell, member: Option<&'a Member>, aimed: bool) -> Element<'a, Message> {
        let metrics = self.metrics;
        let (width, height) = (Length::Fixed(metrics.cell_width), Length::Fixed(metrics.cell_height));

        let frame = move |theme: &Theme| {
            let palette = theme.palette();

            container::Style {
                background: Some(Color { a: EMPTY_FILL, ..palette.text }.into()),
                border: Border::default().rounded(6.0).width(if aimed { 2.0 } else { 1.0 }).color(if aimed {
                    palette.primary
                } else {
                    Color { a: 0.25, ..palette.text }
                }),
                ..container::Style::default()
            }
        };

        let Some(member) = member else {
            return container(Space::new()).width(width).height(height).style(frame).into();
        };

        let face: Element<'a, Message> = self.icons.get(&(member.id, member.form)).map_or_else(
            || container(text(format!("{:03}-{}", member.id, member.form + 1)).size(LEVEL_SIZE)).center_x(Length::Fill).center_y(Length::Fill).into(),
            |handle| iced_image(handle.clone()).width(width).height(height).into(),
        );

        let (level, plus) = match (member.levels(), self.altar) {
            ((level, _), Some(cap)) => (level.min(cap.saturating_add(1)), 0),
            (levels, None) => levels,
        };
        let shown = if plus > 0 { format!("Lv{level}+{plus}") } else { format!("Lv{level}") };
        let badge = container(theme::bold_text(shown).size(metrics.text(BADGE_SIZE)).color(Color::WHITE)).padding([1, 6]).style(|_: &Theme| container::Style {
            background: Some(Color { a: BADGE_FILL, ..Color::BLACK }.into()),
            border: Border::default().rounded(6.0),
            ..container::Style::default()
        });

        let orb_size = ORB_SIZE * metrics.scale;
        let spread = self.worn(member).count().saturating_sub(1) as f32 * orb_size * ORB_STEP + orb_size;
        let mut worn = stack![Space::new().width(Length::Fixed(spread)).height(Length::Fixed(orb_size))];

        for (place, (_, orb)) in self.worn(member).enumerate() {
            let shifted = Padding::default().left(place as f32 * orb_size * ORB_STEP);

            worn = worn.push(container(self.orbs.picture(orb, orb_size)).padding(shifted));
        }

        let coin: Element<'a, Message> = self
            .coin
            .clone()
            .filter(|_| self.priced.get(&cell).is_some_and(|priced| priced.talented))
            .map_or_else(|| Space::new().into(), |handle| iced_image(handle).height(Length::Fixed(COIN_SIZE * metrics.scale)).into());

        let corner = |content: Element<'a, Message>, across: Horizontal, down: Vertical| {
            container(content).width(Length::Fill).height(Length::Fill).align_x(across).align_y(down)
        };

        let priced = self.priced.get(&cell).copied().unwrap_or_default();
        let price = container(theme::bold_text(format!("{}\u{00a2}", priced.cost)).size(metrics.text(COST_SIZE)).color(Color::WHITE))
            .padding([1, 6])
            .style(|_: &Theme| container::Style {
                background: Some(Color { a: BADGE_FILL, ..Color::BLACK }.into()),
                border: Border::default().rounded(6.0),
                ..container::Style::default()
            });
        let shade: Element<'a, Message> = if priced.barred {
            container(Space::new())
                .width(width)
                .height(height)
                .style(|_: &Theme| container::Style {
                    background: Some(Color { a: BARRED_SHADE, ..Color::BLACK }.into()),
                    border: Border::default().rounded(6.0),
                    ..container::Style::default()
                })
                .into()
        } else {
            Space::new().into()
        };

        let tile = stack![
            container(face).width(width).height(height).style(frame),
            shade,
            corner(worn.into(), Horizontal::Left, Vertical::Top),
            corner(coin, Horizontal::Left, Vertical::Bottom),
            corner(badge.into(), Horizontal::Right, Vertical::Top),
            corner(price.into(), Horizontal::Right, Vertical::Bottom),
        ]
            .width(width)
            .height(height);

        let layered = mouse_area(tile).interaction(mouse::Interaction::Grab).on_press(Message::PressCell(cell));

        editor::target(layered, editor::Target::LineupCell(cell))
    }

    fn view_saved<'a>(&'a self, app_state: &'a AppState) -> Element<'a, Message> {
        let roster = &app_state.sandbox.roster;
        let named = roster.current().map_or("", |lineup| lineup.name.as_str());
        let delete = if self.deleting.is_set() { CONFIRM_SHORT_LABEL } else { "Delete" };

        let actions = row![
            button(theme::centered_text("New").size(NAME_SIZE)).width(Length::Fill).style(theme::success_button).on_press(Message::Create),
            button(theme::centered_text(delete).size(NAME_SIZE)).width(Length::Fill).style(theme::danger_button).on_press(Message::Delete),
        ]
            .spacing(6);

        let search = text_input("Search Lineup...", &self.search)
            .on_input(Message::Search)
            .padding(4)
            .size(NAME_SIZE)
            .style(theme::rounded_input);

        let rename = text_input("Lineup Name", named).on_input(Message::Rename).padding(4).size(NAME_SIZE).style(theme::rounded_input);

        let mut listed = Column::new().spacing(4);

        for index in &self.found {
            let Some(lineup) = roster.lineups.get(*index) else {
                continue;
            };

            let label = container(
                row![
                    text(lineup.name.as_str()).size(NAME_SIZE).width(Length::Fill).wrapping(text::Wrapping::None),
                    text(format!("{}/{}", lineup.slots.len(), LINEUP_SLOTS))
                        .size(LEVEL_SIZE)
                        .style(|theme: &Theme| text::Style { color: Some(theme::weak_text_color(theme)) }),
                ]
                    .spacing(6)
                    .align_y(Vertical::Center),
            )
                .padding([6, 8])
                .width(Length::Fill)
                .clip(true);

            listed = listed.push(list_row(label, *index == roster.selected, true, Length::Fill, Message::Load(*index)));
        }

        container(
            column![
                actions,
                search,
                smooth_scroll(scrollable(listed).width(Length::Fill).height(Length::Fill)),
                rename,
            ]
                .spacing(8),
        )
            .width(Length::Fixed(RIGHT_WIDTH))
            .height(Length::Fill)
            .padding(8)
            .style(theme::list_panel_container_right)
            .into()
    }
}
