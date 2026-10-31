use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use kokoro::common::architecture::GAME;
use kokoro::domains::cat::filter::evaluation;
use kokoro::domains::cat::game::stats::{get_final_stats, seeded_level};
use kokoro::domains::cat::scanner::{self, CatEntry};
use kokoro::domains::sandbox::orb::Allowance;
use kokoro::domains::sandbox::{Arrangement, BENCH_SLOTS, Cell, Edge, LINEUP_SLOTS, Lineup, Member, Placed, Roster};
use kokoro::systems::treasure::Bonus;

use crate::domains::cat::{self, FilterPopup, SelectedCat};
use crate::platform::{Picture, Platform};
use crate::systems::context::{self, Shared};
use crate::systems::images::{self, Pictures};
use crate::systems::picture::Pic;
use crate::widget::RowList;

use super::orbs::{self, Book, Shelf};
use super::state::{self, SandboxState};
use super::{Cats, Dialog, Ruled, combos};

const RIPE_MS: f64 = 160.0;
const DOUBLE_CLICK_MS: f64 = 400.0;
const SLACK: f64 = 6.0;
const FIRST_TALENT_FORM: usize = 2;
const COLUMNS: usize = 5;
const SMALLEST: f32 = 0.45;
const TEXT_FLOOR: f32 = 0.8;
const DECK_SHARE: f32 = 0.68;
const CARD_GAP: f32 = 10.0;
const LEVEL_SIZE: f32 = 11.0;
const BADGE_SIZE: f32 = 12.0;
const COST_SIZE: f32 = 12.0;
const ORB_SIZE: f32 = 26.0;
const COIN_SIZE: f32 = 18.0;
const ORB_STEP: f32 = 0.55;
const NP_ICON: &str = "gatyaitemD_07_f.png";
const SLOT_ORB_SIZE: f32 = 44.0;
const UNIT_WIDTH: f32 = 760.0;
const ORB_WIDTH: f32 = 310.0;
const CONFIRM_MS: u32 = 2000;
const CONFIRM_LABEL: &str = "Sure?";

#[derive(Clone, Copy, PartialEq)]
struct Metrics {
    scale: f32,
    pad: f32,
    cell_width: f32,
    cell_height: f32,
    gap: f32,
    header: f32,
    header_gap: f32,
    bench_gap: f32,
    bench_title: f32,
    bench_note: f32,
}

impl Metrics {
    const FULL: Self = Self::at(1.0);

    const fn at(scale: f32) -> Self {
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

    fn fitting(width: f32, height: f32) -> Self {
        if width <= 0.0 || height <= 0.0 {
            return Self::FULL;
        }

        let across = (width - CARD_GAP * 2.0) / Self::FULL.deck_width();
        let down = height * DECK_SHARE / Self::FULL.deck_height();

        Self::at(across.min(down).clamp(SMALLEST, 1.0))
    }

    fn grid_width(&self) -> f32 {
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

    fn text(&self, size: f32) -> f32 {
        size * self.scale.max(TEXT_FLOOR)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Cargo {
    Fresh(u32),
    Held(Cell),
}

#[derive(Clone, Copy, PartialEq)]
enum Landing {
    Off,
    Card,
    Cell(Cell, Edge),
}

#[derive(Clone, Copy, PartialEq, Default)]
enum Drag {
    #[default]
    Idle,
    Pressed { cargo: Cargo, since: f64, from: (f64, f64) },
    Moving { cargo: Cargo, lifted: Option<Cell>, at: (f64, f64), landing: Landing },
}

#[derive(Clone, Copy, Default)]
struct Priced {
    cost: i32,
    barred: bool,
    talented: bool,
}

#[derive(Clone, PartialEq)]
struct Editing {
    platform: Platform,
    sandbox: Signal<SandboxState>,
}

impl Editing {
    fn roster(&self, change: impl FnOnce(&mut Roster)) {
        state::apply(&self.platform, self.sandbox, |next| {
            change(&mut next.roster);
            next.roster.remember(GAME);
        });
    }
}

fn shown_form(cat: &CatEntry, banner_form: usize) -> usize {
    let highest = banner_form.min(cat.forms.len() - 1);

    (0..=highest)
        .rev()
        .find(|form| cat.forms[*form] && cat.banner_paths[*form].is_some())
        .or_else(|| (0..=highest).rev().find(|form| cat.forms[*form]))
        .or_else(|| cat.forms.iter().position(|exists| *exists))
        .unwrap_or(0)
}

fn landing(at: (f64, f64), deck: Option<(f64, f64)>, metrics: Metrics) -> Landing {
    let Some((left, top)) = deck else {
        return Landing::Card;
    };
    let (x, y) = (at.0 as f32 - left as f32, at.1 as f32 - top as f32);

    if !(0.0..metrics.deck_width()).contains(&x) || !(0.0..metrics.deck_height()).contains(&y) {
        return Landing::Off;
    }

    let half = metrics.gap / 2.0;
    let across = x - metrics.pad + half;
    let pitch = metrics.cell_width + metrics.gap;

    if !(0.0..pitch * COLUMNS as f32).contains(&across) {
        return Landing::Card;
    }

    let column = (across / pitch) as usize;
    let offset = across - column as f32 * pitch;
    let edge = if offset < pitch / 2.0 { Edge::Before } else { Edge::After };
    let fall = metrics.cell_height + metrics.gap;
    let bench = y - metrics.bench_top() + half;

    if (0.0..fall).contains(&bench) {
        return Landing::Cell(Cell::Bench(column), edge);
    }

    let grid = y - metrics.grid_top() + half;
    let lines = LINEUP_SLOTS / COLUMNS;

    if !(0.0..fall * lines as f32).contains(&grid) {
        return Landing::Card;
    }

    let line = (grid / fall) as usize;

    Landing::Cell(Cell::Slot(line * COLUMNS + column), edge)
}

fn plan(lifted: Option<Cell>, landing: Landing, lineup: Option<&Lineup>) -> Option<Arrangement> {
    let Landing::Cell(target, edge) = landing else {
        return None;
    };

    lineup?.arrange(lifted, target, edge)
}

fn price(lineup: &Lineup, cats: &[CatEntry], rules: &Ruled) -> HashMap<Cell, Priced> {
    let mut priced = HashMap::new();
    let slots = lineup.slots.iter().enumerate().map(|(index, member)| (Cell::Slot(index), member));
    let bench = lineup.bench.iter().enumerate().filter_map(|(index, member)| Some((Cell::Bench(index), member.as_ref()?)));
    let mut fielded: Vec<usize> = Vec::new();

    for (cell, member) in slots.chain(bench) {
        let cat = cats.iter().find(|cat| cat.id == member.id);
        let stats = cat.and_then(|cat| cat.stats.get(member.form).and_then(Option::as_ref));
        let talents = (member.form >= FIRST_TALENT_FORM).then_some(&member.talents);
        let (level, plus) = member.levels();
        let base = stats.map_or(0, |stats| {
            get_final_stats(stats, cat.and_then(|cat| cat.curve.as_ref()), (level + plus) as i32, cat.and_then(|cat| cat.talent_data.as_ref()), talents, &Bonus::default()).eoc1_cost
        });
        let rarity = cat.and_then(|cat| usize::try_from(cat.unitbuy.rarity).ok()).unwrap_or(0);
        let cost = rules.0.cost(base, rarity);
        let groups = cat.and_then(|cat| cat.talent_data.as_ref()).map_or(0, |talent| talent.groups.len());
        let talented = member.talented() && member.talents.iter().any(|(index, level)| *level > 0 && usize::from(*index) < groups);
        let earlier = fielded.iter().filter(|held| **held == rarity).count();
        let slot = match cell {
            Cell::Slot(slot) => Some(slot),
            Cell::Bench(_) => None,
        };

        priced.insert(cell, Priced { cost, talented, barred: rules.0.bars(member.id, rarity, cost, earlier, slot) });

        if matches!(cell, Cell::Slot(_)) {
            fielded.push(rarity);
        }
    }

    priced
}

#[derive(Clone)]
struct Recruiter {
    cats: Rc<Vec<CatEntry>>,
    orb_slots: Arc<HashMap<u32, usize>>,
    banner_form: usize,
}

impl Recruiter {
    fn recruit(&self, id: u32, form: Option<usize>, roster: &Roster, settings: &kokoro::domains::settings::Settings) -> Option<Member> {
        let cat = self.cats.iter().find(|cat| cat.id == id)?;
        let lowest = shown_form(cat, self.banner_form);
        let level = seeded_level(cat, settings).1;
        let fresh = Member { id, form: form.unwrap_or(lowest), level, talents: HashMap::new(), orbs: Vec::new() };

        Some(self.fitted(roster.dress(fresh, form, GAME)))
    }

    fn fitted(&self, mut member: Member) -> Member {
        member.orbs.resize(self.orb_slots.get(&member.id).copied().unwrap_or(0), None);
        member
    }
}

#[component]
pub(crate) fn Editor(cats: Cats, globals: Shared, rules: Ruled, altar: Option<u32>) -> Element {
    let platform = use_context::<Platform>();
    let settings = context::settings();
    let sandbox = state::state();
    let pictures = images::pictures();
    let mut cat_state = cat::provide();
    let mut drag = use_signal(Drag::default);
    let mut plan_held: Signal<Option<Arrangement>> = use_signal(|| None);
    let mut editing: Signal<Option<u32>> = use_signal(|| None);
    let mut orb_slot: Signal<Option<usize>> = use_signal(|| None);
    let mut search = use_signal(String::new);
    let mut deleting = use_signal(|| false);
    let mut delete_round = use_signal(|| 0u32);
    let mut deck_origin: Signal<Option<(f64, f64)>> = use_signal(|| None);
    let mut deck_mount: Signal<Option<MountedEvent>> = use_signal(|| None);
    let mut area = use_signal(|| (0f32, 0f32));
    let mut clicked: Signal<Option<(Cell, f64)>> = use_signal(|| None);
    let shared = globals.clone();
    let shelf = use_memo(use_reactive!(|shared| Shelf(Rc::new(Book::load(&shared)))));
    let shelf = shelf();
    let editing_tools = Editing { platform: platform.clone(), sandbox };
    let held_icons: Rc<RefCell<HashMap<u32, Rc<str>>>> = use_hook(Rc::default);

    use_hook(move || {
        let query = sandbox.peek().search_query.clone();

        cat_state.search.set(query);
    });

    let vault = globals.0.vault();
    let banner_form = settings.read().sandbox.banner_form;
    let recruiter = Recruiter { cats: Rc::clone(&cats.0), orb_slots: vault.vds.cats.orb_slots(&vault.vfs), banner_form };
    let held = sandbox.read();
    let lineup = Rc::new(held.roster.current().cloned().unwrap_or_default());
    let roster_names: Vec<(usize, String, usize)> = held.roster.lineups.iter().enumerate().map(|(index, lineup)| (index, lineup.name.clone(), lineup.slots.len())).collect();
    let selected_lineup = held.roster.selected;

    drop(held);

    let priced = Rc::new(price(&lineup, &cats.0, &rules));
    let (area_width, area_height) = area();
    let metrics = Metrics::fitting(area_width, area_height);
    let dragging = drag();
    let lifted = match dragging {
        Drag::Moving { lifted, .. } => lifted,
        _ => None,
    };
    let arrangement = plan_held();
    let coin = vault.vfs.find(NP_ICON).and_then(|path| pictures.trimmed(&path)).map(|sized| sized.url);
    let icon_of = {
        let pictures = pictures.clone();
        let cats = Rc::clone(&cats.0);
        let globals = globals.clone();

        let held_icons = Rc::clone(&held_icons);

        move |id: u32, form: usize| -> Option<Rc<str>> {
            let vfs = &globals.0.vault().vfs;
            let path = match cats.iter().find(|cat| cat.id == id) {
                Some(cat) => scanner::deploy_icon(vfs, cat, form),
                None => scanner::orphan_icon(vfs, id, form),
            };
            let url = path.and_then(|path| pictures.trimmed(&path)).map(|sized| sized.url);

            match url {
                Some(url) => {
                    held_icons.borrow_mut().insert(id, Rc::clone(&url));

                    Some(url)
                }
                None => held_icons.borrow().get(&id).cloned(),
            }
        }
    };
    let refresh_deck = move || {
        if let Some(mounted) = deck_mount.peek().clone() {
            spawn(async move {
                if let Ok(rect) = mounted.get_client_rect().await {
                    deck_origin.set(Some((rect.origin.x, rect.origin.y)));
                }
            });
        }
    };
    let press = {
        let platform = platform.clone();

        move |cargo: Cargo, at: (f64, f64)| {
            let now = platform.host().now();

            refresh_deck();
            drag.set(Drag::Pressed { cargo, since: now, from: at });
        }
    };
    let moved = {
        let platform = platform.clone();

        Callback::new(move |at: (f64, f64)| {
            if *drag.peek() == Drag::Idle {
                return;
            }

            let now = platform.host().now();
            let held = sandbox.peek();
            let lineup = held.roster.current();
            let origin = *deck_origin.peek();
            let next = match *drag.peek() {
                Drag::Pressed { cargo, since, from } => {
                    let strayed = (at.0 - from.0).abs() > SLACK || (at.1 - from.1).abs() > SLACK;

                    if strayed || now - since >= RIPE_MS {
                        let landing = landing(at, origin, metrics);
                        let lifted = match cargo {
                            Cargo::Held(cell) => Some(cell),
                            Cargo::Fresh(id) => lineup.and_then(|lineup| lineup.find(id)),
                        };

                        plan_held.set(plan(lifted, landing, lineup));

                        Drag::Moving { cargo, lifted, at, landing }
                    } else {
                        Drag::Pressed { cargo, since, from }
                    }
                }
                Drag::Moving { cargo, lifted, landing: before, .. } => {
                    let landing = landing(at, origin, metrics);

                    if landing != before {
                        plan_held.set(plan(lifted, landing, lineup));
                    }

                    Drag::Moving { cargo, lifted, at, landing }
                }
                Drag::Idle => Drag::Idle,
            };

            drag.set(next);
        })
    };
    let released = {
        let platform = platform.clone();
        let recruiter = recruiter.clone();
        let tools = editing_tools.clone();
        let cats = Rc::clone(&cats.0);

        Callback::new(move |()| {
            if *drag.peek() == Drag::Idle {
                return;
            }

            let settled = std::mem::take(&mut *drag.write());
            let plan = plan_held.take();
            let now = platform.host().now();

            match settled {
                Drag::Pressed { cargo: Cargo::Fresh(id), .. } => {
                    let settings = settings.read();

                    tools.roster(|roster| {
                        if let Some(member) = recruiter.recruit(id, None, roster, &settings) {
                            roster.current_mut().add(member);
                        }
                    });
                }
                Drag::Pressed { cargo: Cargo::Held(cell), .. } => {
                    let doubled = clicked.peek().is_some_and(|(held, at)| held == cell && now - at <= DOUBLE_CLICK_MS);

                    clicked.set((!doubled).then_some((cell, now)));

                    if doubled {
                        let member = sandbox.peek().roster.current().and_then(|lineup| lineup.get(cell).cloned());

                        if let Some(member) = member
                            && let Some(cat) = cats.iter().find(|cat| cat.id == member.id)
                        {
                            cat_state.seed(cat, member.form, &member.level, &member.talents);
                            editing.set(Some(member.id));
                            orb_slot.set(None);
                        }
                    }
                }
                Drag::Moving { cargo, landing: Landing::Cell(..), .. } => {
                    let Some(plan) = plan else {
                        return;
                    };
                    let settings = settings.read();

                    tools.roster(|roster| {
                        let fresh = match cargo {
                            Cargo::Fresh(id) if plan.lifted.is_none() => recruiter.recruit(id, None, roster, &settings),
                            _ => None,
                        };

                        if plan.lifted.is_some() || fresh.is_some() {
                            roster.current_mut().commit(&plan, fresh);
                        }
                    });
                }
                Drag::Moving { cargo: Cargo::Held(from), landing: Landing::Off, .. } => {
                    tools.roster(|roster| {
                        roster.current_mut().take(from);
                    });

                    if editing.peek().is_some_and(|id| sandbox.peek().roster.current().is_none_or(|lineup| lineup.find(id).is_none())) {
                        editing.set(None);
                        orb_slot.set(None);
                    }
                }
                _ => {}
            }
        })
    };
    let cancelled = Callback::new(move |()| {
        drag.set(Drag::Idle);
        plan_held.set(None);
    });
    let shown = |cell: Cell| -> (Option<(Cell, Member)>, bool) {
        let source = arrangement.as_ref().map_or_else(|| (lifted != Some(cell)).then_some(Placed::Kept(cell)), |plan| plan.get(cell));

        match source {
            Some(Placed::Kept(source)) => (lineup.get(source).map(|member| (source, member.clone())), false),
            Some(Placed::Carried) => (None, true),
            None => (None, false),
        }
    };
    let cell_view = |cell: Cell| -> Element {
        let (member, aimed) = shown(cell);
        let frame_class = if aimed { "deck-cell aimed" } else { "deck-cell" };
        let size_style = format!("width: {}px; height: {}px;", metrics.cell_width, metrics.cell_height);
        let mut press = press.clone();

        let Some((source, member)) = member else {
            return rsx! { div { key: "{cell:?}", class: "{frame_class}", style: "{size_style}" } };
        };
        let icon = icon_of(member.id, member.form);
        let (level, plus) = match (member.levels(), altar) {
            ((level, _), Some(cap)) => (level.min(cap.saturating_add(1)), 0),
            (levels, None) => levels,
        };
        let badge = if plus > 0 { format!("Lv{level}+{plus}") } else { format!("Lv{level}") };
        let priced_cell = priced.get(&source).copied().unwrap_or_default();
        let worn: Vec<u32> = member.worn(recruiter.orb_slots.get(&member.id).copied().unwrap_or(0), &shelf.0.orbs).map(|(_, orb)| orb).collect();
        let orb_size = ORB_SIZE * metrics.scale;
        let fallback = format!("{:03}-{}", member.id, member.form + 1);
        let coin = coin.clone().filter(|_| priced_cell.talented);

        rsx! {
            div {
                key: "{cell:?}",
                class: "{frame_class} filled",
                style: "{size_style}",
                onpointerdown: move |event| {
                    if event.data().trigger_button() != Some(MouseButton::Primary) {
                        return;
                    }

                    event.prevent_default();
                    let point = event.data().client_coordinates();

                    press(Cargo::Held(cell), (point.x, point.y));
                },
                onpointermove: move |event| {
                    if *drag.peek() != Drag::Idle {
                        let point = event.data().client_coordinates();

                        moved.call((point.x, point.y));
                    }
                },
                onpointerup: move |_| {
                    if *drag.peek() != Drag::Idle {
                        released.call(());
                    }
                },
                onpointercancel: move |_| {
                    if *drag.peek() != Drag::Idle {
                        cancelled.call(());
                    }
                },
                match icon {
                    Some(url) => rsx! { Pic { url, class: "deck-face" } },
                    None => rsx! { div { class: "deck-fallback", style: "font-size: {LEVEL_SIZE}px;", "{fallback}" } },
                }
                if priced_cell.barred {
                    div { class: "deck-shade" }
                }
                div { class: "deck-orbs", style: "height: {orb_size}px;",
                    for (place, orb) in worn.iter().enumerate() {
                        div { key: "{place}", class: "deck-orb", style: "left: {place as f32 * orb_size * ORB_STEP}px;",
                            {orbs::picture(&pictures, &shelf.0.orbs, *orb, orb_size)}
                        }
                    }
                }
                if let Some(coin) = coin {
                    Pic { url: coin, class: "deck-coin", style: format!("height: {}px;", COIN_SIZE * metrics.scale) }
                }
                div { class: "deck-badge deck-level", style: "font-size: {metrics.text(BADGE_SIZE)}px;", "{badge}" }
                div { class: "deck-badge deck-cost", style: "font-size: {metrics.text(COST_SIZE)}px;", "{priced_cell.cost}\u{00a2}" }
            }
        }
    };
    let ghost = match dragging {
        Drag::Moving { cargo, at, .. } => {
            let url = match cargo {
                Cargo::Held(cell) => lineup.get(cell).and_then(|member| icon_of(member.id, member.form)),
                Cargo::Fresh(id) => cats.0.iter().find(|cat| cat.id == id).and_then(|cat| {
                    let shown = shown_form(cat, banner_form);
                    let form = sandbox.peek().roster.recall(id, GAME).map_or(shown, |past| past.form);

                    icon_of(id, form).or_else(|| icon_of(id, shown))
                }),
            };

            Some((url, at))
        }
        _ => None,
    };
    let roster_cats = Rc::clone(&cats.0);
    let mut press_row = press.clone();
    let focus = editing().and_then(|id| lineup.find(id)).and_then(|cell| lineup.get(cell)).map(|member| (member.id, member.form));
    let adopt_tools = editing_tools.clone();
    let adopt_recruiter = recruiter.clone();
    let adopt_globals = globals.clone();
    let onadopt = move |line: usize| {
        let vault = adopt_globals.0.vault();
        let rows = vault.vds.cats.combos(&vault.vfs);
        let Some(row) = rows.get(line) else {
            return;
        };
        let settings = settings.read();

        adopt_tools.roster(|roster| {
            let mut lineup = std::mem::take(roster.current_mut());

            lineup.adopt(row, |id, form| adopt_recruiter.recruit(id, Some(form), roster, &settings).unwrap_or_else(|| adopt_recruiter.fitted(Member { id, form, ..Member::default() })));
            *roster.current_mut() = lineup;
        });
    };
    let editing_id = editing();
    let edited = editing_id.and_then(|id| lineup.find(id)).and_then(|cell| lineup.get(cell).cloned().map(|member| (cell, member)));
    let edited_cat = edited.as_ref().and_then(|(_, member)| cats.0.iter().find(|cat| cat.id == member.id).cloned().map(Rc::new));
    let delete_label = if deleting() { CONFIRM_LABEL } else { "Delete" };
    let wanted = search.read().to_lowercase();
    let found: Vec<(usize, String, usize)> = roster_names.into_iter().filter(|(_, name, _)| wanted.is_empty() || name.to_lowercase().contains(&wanted)).collect();
    let named = lineup.name.clone();
    let filter_active = cat_state.filter.read().is_active();
    let filter_class = if filter_active { "btn btn-toggle active roster-filter" } else { "btn btn-toggle roster-filter" };
    let roster_tools = editing_tools.clone();
    let rename_tools = editing_tools.clone();
    let create_tools = editing_tools.clone();
    let delete_tools = editing_tools.clone();
    let delete_platform = platform.clone();

    rsx! {
        div { class: "lineup-editor",
            if dragging != Drag::Idle {
                div {
                    class: "drag-scrim",
                    onpointermove: move |event| {
                        let point = event.data().client_coordinates();

                        moved.call((point.x, point.y));
                    },
                    onpointerup: move |_| released.call(()),
                    onpointercancel: move |_| cancelled.call(()),
                    onpointerleave: move |_| cancelled.call(()),
                }
            }
            if let Some((url, (x, y))) = ghost {
                div { class: "lineup-ghost", style: "left: {x as f32 - metrics.cell_width / 2.0}px; top: {y as f32 - metrics.cell_height / 2.0}px; width: {metrics.cell_width}px; height: {metrics.cell_height}px;",
                    if let Some(url) = url {
                        Pic { url, class: "deck-face" }
                    }
                }
            }
            SandboxRoster { cats: cats.clone(), banner_form, filter_class: filter_class.to_owned(), onpress: Callback::new(move |(id, at): (u32, (f64, f64))| press_row(Cargo::Fresh(id), at)), onmove: moved, onrelease: released, oncancel: cancelled, onsearch: Callback::new(move |query: String| { let sandbox = roster_tools.sandbox; state::apply(&roster_tools.platform, sandbox, |next| next.search_query = query); }) }
            div {
                class: "lineup-center",
                onresize: move |event| {
                    if let Ok(size) = event.data().get_content_box_size() {
                        area.set((size.width as f32, size.height as f32));
                    }
                },
                div {
                    class: "outlined-card deck-card",
                    style: "width: {metrics.deck_width()}px; height: {metrics.deck_height()}px; padding: {metrics.pad}px;",
                    onmounted: move |event| deck_mount.set(Some(event)),
                    div { class: "deck-title bold", style: "height: {metrics.header}px; font-size: {metrics.text(16.0)}px;", "Lineup" }
                    div { style: "height: {metrics.header_gap}px;" }
                    div { class: "deck-grid", style: "gap: {metrics.gap}px;",
                        for line in 0..LINEUP_SLOTS / COLUMNS {
                            div { key: "{line}", class: "deck-row", style: "gap: {metrics.gap}px;",
                                for column in 0..COLUMNS {
                                    {cell_view(Cell::Slot(line * COLUMNS + column))}
                                }
                            }
                        }
                    }
                    div { style: "height: {metrics.bench_gap}px;" }
                    div { class: "deck-title bold", style: "height: {metrics.bench_title}px; font-size: {metrics.text(14.0)}px;", "Bench" }
                    div { class: "deck-note weak", style: "height: {metrics.bench_note}px; font-size: {metrics.text(LEVEL_SIZE)}px;", "Does not affect battle" }
                    div { style: "height: {metrics.gap}px;" }
                    div { class: "deck-row", style: "gap: {metrics.gap}px;",
                        for index in 0..BENCH_SLOTS {
                            {cell_view(Cell::Bench(index))}
                        }
                    }
                }
                combos::Combos { cats: cats.clone(), globals: globals.clone(), rules: rules.clone(), lineup: combos::Held(Rc::clone(&lineup)), focus, onadopt }
            }
            div { class: "list-panel-right saved-panel",
                div { class: "saved-actions",
                    button { class: "btn btn-success saved-btn", onclick: move |_| {
                        create_tools.roster(Roster::create);
                        editing.set(None);
                        orb_slot.set(None);
                    }, "New" }
                    button { class: "btn btn-danger saved-btn", onclick: move |_| {
                        if !deleting() {
                            let round = delete_round() + 1;

                            delete_round.set(round);
                            deleting.set(true);

                            let sleeper = delete_platform.clone();

                            spawn(async move {
                                sleeper.host().sleep(CONFIRM_MS).await;

                                if delete_round() == round {
                                    deleting.set(false);
                                }
                            });

                            return;
                        }

                        deleting.set(false);
                        delete_tools.roster(Roster::delete);
                        editing.set(None);
                        orb_slot.set(None);
                    }, "{delete_label}" }
                }
                input { class: "input saved-search", r#type: "search", placeholder: "Search Lineup...", value: "{search}", oninput: move |event| search.set(event.value()) }
                div { class: "saved-list scroll",
                    for (index, name, count) in found {
                        {
                            let tools = editing_tools.clone();
                            let class = if index == selected_lineup { "list-row filled selected saved-row" } else { "list-row filled saved-row" };

                            rsx! {
                                button { key: "{index}", class: "{class}", onclick: move |_| {
                                    tools.roster(|roster| {
                                        if index < roster.lineups.len() {
                                            roster.selected = index;
                                        }
                                    });
                                    deleting.set(false);
                                    editing.set(None);
                                    orb_slot.set(None);
                                },
                                    span { class: "saved-name", "{name}" }
                                    span { class: "weak saved-count", "{count}/{LINEUP_SLOTS}" }
                                }
                            }
                        }
                    }
                }
                input { class: "input saved-rename", r#type: "text", placeholder: "Lineup Name", value: "{named}", oninput: move |event| rename_tools.roster(|roster| roster.current_mut().name = event.value()) }
            }
        }
        FilterPopup { cats: cat::list::Entries(roster_cats), globals: globals.clone() }
        if let (Some((cell, member)), Some(cat)) = (edited.clone(), edited_cat) {
            UnitPopup { cat: SelectedCat(cat), member, cell, globals: globals.clone(), shelf: shelf.clone(), tools: editing_tools.clone(), orb_slot, onclose: move |()| {
                editing.set(None);
                orb_slot.set(None);
            } }
        }
    }
}

#[component]
fn SandboxRoster(
    cats: Cats,
    banner_form: usize,
    filter_class: String,
    onpress: Callback<(u32, (f64, f64))>,
    onmove: Callback<(f64, f64)>,
    onrelease: Callback<()>,
    oncancel: Callback<()>,
    onsearch: Callback<String>,
) -> Element {
    let mut cat_state = cat::state();
    let pictures = images::pictures();
    let search = (cat_state.search)();
    let shown = use_memo(use_reactive!(|(cats, banner_form)| {
        let query = cat_state.search.read().to_lowercase();
        let filter = cat_state.filter.read();

        Rc::new(
            cats.0
                .iter()
                .filter(|cat| evaluation::entity_passes_filter(cat, &filter))
                .filter(|cat| query.is_empty() || cat.base_id_str().contains(&query) || cat.names.iter().flatten().any(|name| name.to_lowercase().contains(&query)))
                .map(|cat| {
                    let form = shown_form(cat, banner_form);

                    (cat.id, cat.banner_paths.get(form).cloned().flatten().or_else(|| cat.image_path.clone()))
                })
                .collect::<Vec<(u32, Option<PathBuf>)>>(),
        )
    }));
    let shown: Rc<Vec<(u32, Option<PathBuf>)>> = shown();
    let frame_style = pictures.frame_url().map_or_else(String::new, |url| format!("--banner-frame: url({url});"));
    let count = shown.len();
    let row = Callback::new(move |index: usize| {
        let Some((id, path)) = shown.get(index).cloned() else {
            return rsx! {};
        };

        rsx! {
            RosterRow { key: "{id}", id, path, onpress, onmove, onrelease, oncancel }
        }
    });

    rsx! {
        div { class: "roster-panel list-panel sandbox-roster", style: "{frame_style}",
            button { class: "{filter_class}", onclick: move |_| cat_state.filter_open.set(true), "Filter" }
            input {
                class: "input roster-search",
                r#type: "search",
                placeholder: "Search Cat...",
                value: "{search}",
                oninput: move |event| {
                    cat_state.search.set(event.value());
                    onsearch.call(event.value());
                },
            }
            RowList { count, row, empty: "No Cats Found!" }
        }
    }
}

#[component]
fn RosterRow(id: u32, path: Option<PathBuf>, onpress: Callback<(u32, (f64, f64))>, onmove: Callback<(f64, f64)>, onrelease: Callback<()>, oncancel: Callback<()>) -> Element {
    let pictures = images::pictures();
    let banner = path.as_ref().and_then(|path| pictures.banner(Picture::CatBanner, path));
    let wanted = path.as_ref().and_then(|path| Pictures::banner_key(&Picture::CatBanner, path));
    let forget = pictures.clone();

    use_drop(move || {
        if let Some(key) = &wanted {
            forget.unwant(key);
        }
    });

    rsx! {
        div {
            class: "list-row roster-row grabbable",
            onpointerdown: move |event| {
                if event.data().trigger_button() != Some(MouseButton::Primary) {
                    return;
                }

                event.prevent_default();
                let point = event.data().client_coordinates();

                onpress.call((id, (point.x, point.y)));
            },
            onpointermove: move |event| {
                let point = event.data().client_coordinates();

                onmove.call((point.x, point.y));
            },
            onpointerup: move |_| onrelease.call(()),
            onpointercancel: move |_| oncancel.call(()),
            div { class: "banner-slot",
                if let Some(url) = banner {
                    Pic { url, class: "banner" }
                }
            }
        }
    }
}

#[component]
fn UnitPopup(cat: SelectedCat, member: Member, cell: Cell, globals: Shared, shelf: Shelf, tools: Editing, orb_slot: Signal<Option<usize>>, onclose: EventHandler<()>) -> Element {
    let cat_state = cat::state();
    let pictures = images::pictures();
    let title = cat.0.display_name(member.form);
    let form = (cat_state.form)();
    let watched = cat.clone();
    let slots = tools.sandbox.peek().roster.current().and_then(|lineup| lineup.get(cell)).map_or(0, |member| member.orbs.len());
    let orbs_now = member.orbs.clone();
    let writer = tools.clone();

    use_effect(move || {
        let form = (cat_state.form)();
        let level = (cat_state.level_input)();
        let talents = cat_state.levels_for(watched.0.id);
        let current = writer.sandbox.peek().roster.current().and_then(|lineup| lineup.get(cell).cloned());
        let Some(current) = current else {
            return;
        };

        if current.form == form && current.level == level && current.talents == talents {
            return;
        }

        writer.roster(|roster| {
            if let Some(member) = roster.current_mut().get_mut(cell) {
                member.form = form;
                member.level = level;
                member.talents = talents;
            }
        });
    });

    let allowance = Allowance::of(&cat.0);
    let header = (form >= FIRST_TALENT_FORM && slots > 0).then(|| {
        rsx! {
            div { class: "orb-header",
                span { class: "bold orb-header-title", "Orbs" }
                div { class: "orb-slots",
                    for (slot, orb) in orbs_now.iter().enumerate() {
                        button { key: "{slot}", class: "btn btn-neutral orb-slot", onclick: move |_| orb_slot.set(Some(slot)),
                            match orb {
                                Some(orb) => orbs::picture(&pictures, &shelf.0.orbs, *orb, SLOT_ORB_SIZE),
                                None => rsx! { span { class: "orb-plus", "+" } },
                            }
                        }
                    }
                }
            }
        }
    });
    let picking = orb_slot();
    let held = picking.and_then(|slot| member.orbs.get(slot).copied().flatten());
    let pick_tools = tools.clone();

    rsx! {
        Dialog { title, width: UNIT_WIDTH, class: "unit-popup", onclose: move |()| onclose.call(()),
            cat::Detail { cat: cat.clone(), globals: globals.clone(), extra: header }
        }
        if let Some(slot) = picking {
            Dialog { title: "Orb", width: ORB_WIDTH, onclose: move |()| orb_slot.set(None),
                orbs::Picker { shelf: shelf.clone(), held, allowance, onpick: move |orb: Option<u32>| {
                    pick_tools.roster(|roster| {
                        if let Some(member) = roster.current_mut().get_mut(cell)
                            && let Some(target) = member.orbs.get_mut(slot)
                        {
                            *target = orb;
                        }
                    });
                } }
            }
        }
    }
}
