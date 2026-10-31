pub(crate) mod filter;
mod list;

use std::rc::Rc;

use dioxus::prelude::*;

use codex::combat::{Entity, Faction};
use kokoro::domains::enemy::filter::EnemyFilterState;
use kokoro::domains::enemy::animation as enemy_animation;
use kokoro::domains::enemy::scanner::{self, EnemyEntry};
use kokoro::domains::enemy::statblock::{self as enemy_statblock, Subject};
use kokoro::domains::settings::Settings;
use kokoro::systems::combat::RenderContext;
use kokoro::systems::combat::abilities::collect_ability_data;
use kokoro::systems::combat::registry::{
    Magnification, STAT_ATK_CYCLE, STAT_ATTACK, STAT_CASH_DROP, STAT_DPS, STAT_HITPOINTS, STAT_KNOCKBACKS, STAT_RANGE, STAT_SPEED, StatContext, format_stat,
};

use crate::app::{self, Route};
use crate::domains::cat::icons::{self, IMG015};
use crate::domains::cat::{abilities, header_class};
use crate::platform::Platform;
use crate::systems::catalog::{self, Load};
use crate::systems::context::{self, Shared};
use crate::systems::export::{self, Statblock};
use crate::systems::images::{self, Pictures};
use crate::systems::library::Library;
use crate::systems::picture::Pic;
use crate::widget::{Hold, NameBox, Settled, StatFrames, StatHeader, StatValue, Status};

pub(crate) use filter::FilterPopup;

const EMPTY_ICON: &str = "uni.png";
const LAST_ENEMY_KEY: &str = "last-enemy";
const DEFAULT_MAGNIFICATION: &str = "100";

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum DetailTab {
    #[default]
    Abilities,
    Details,
    Animation,
}

const ICON_BOX_WIDTH: f32 = 110.0;
const ICON_BOX_HEIGHT: f32 = 96.0;
const TABS: [(DetailTab, &str); 3] = [(DetailTab::Abilities, "Abilities"), (DetailTab::Details, "Details"), (DetailTab::Animation, "Animation")];

#[derive(Clone, Copy)]
pub(crate) struct EnemyState {
    pub(crate) search: Signal<String>,
    pub(crate) filter: Signal<EnemyFilterState>,
    pub(crate) filter_open: Signal<bool>,
    pub(crate) tab: Signal<DetailTab>,
    pub(crate) mag_input: Signal<String>,
    pub(crate) magnification: Signal<Magnification>,
    pub(crate) seeded_for: Signal<Option<u32>>,
    pub(crate) appearances: Signal<Option<u32>>,
}

pub(crate) fn provide() -> EnemyState {
    use_context_provider(|| EnemyState {
        search: Signal::new(String::new()),
        filter: Signal::new(EnemyFilterState::default()),
        filter_open: Signal::new(false),
        tab: Signal::new(DetailTab::Abilities),
        mag_input: Signal::new(DEFAULT_MAGNIFICATION.to_owned()),
        magnification: Signal::new(Magnification::default()),
        seeded_for: Signal::new(None),
        appearances: Signal::new(None),
    })
}

pub(crate) fn state() -> EnemyState {
    use_context::<EnemyState>()
}

fn typable_magnification(value: &str) -> bool {
    value.chars().all(|glyph| glyph.is_ascii_digit() || matches!(glyph, '/' | '|' | '\\'))
}

pub(crate) fn parse_magnification(input: &str) -> Magnification {
    let trimmed = input.trim();
    let parts: Vec<&str> = trimmed.split(['/', '|', '\\']).collect();

    if parts.len() >= 2 {
        let hitpoints = parts[0].trim().parse::<i32>().unwrap_or(100);
        let attack = parts[1].trim().parse::<i32>().unwrap_or(hitpoints);

        Magnification { hitpoints, attack }
    } else {
        let magnification = trimmed.parse::<i32>().unwrap_or(100);

        Magnification { hitpoints: magnification, attack: magnification }
    }
}

impl EnemyState {
    fn select(&mut self, id: u32) {
        if (self.seeded_for)() == Some(id) {
            return;
        }

        self.seeded_for.set(Some(id));
        self.mag_input.set(DEFAULT_MAGNIFICATION.to_owned());
        self.magnification.set(Magnification::default());
    }

    pub(crate) fn set_magnification(&mut self, input: String) {
        if !typable_magnification(&input) {
            return;
        }

        self.magnification.set(parse_magnification(&input));
        self.mag_input.set(input);
    }
}

#[derive(Clone)]
pub(crate) struct SelectedEnemy(pub(crate) Rc<EnemyEntry>);

impl PartialEq for SelectedEnemy {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[component]
pub(crate) fn Enemies(segments: Vec<String>) -> Element {
    app::use_last_visited(LAST_ENEMY_KEY, &segments, |segments| Route::Enemies { segments });

    rsx! {
        Browser { selected: Route::picked(&segments) }
    }
}

#[component]
fn Browser(selected: Option<u32>) -> Element {
    let platform = use_context::<Platform>();
    let library = use_context::<Library>();
    let catalogs = catalog::catalogs();
    let settings = context::settings();
    let mut state = state();
    let navigator = use_navigator();
    let visitor = platform.clone();

    use_effect(use_reactive!(|selected| {
        let (Some(id), Load::Ready(enemies)) = (selected, catalogs.enemies()) else {
            return;
        };

        if enemies.iter().any(|enemy| enemy.id == id) {
            app::remember_visit(&visitor, LAST_ENEMY_KEY, &Route::enemy(id));
        } else {
            app::forget_visit(&visitor, navigator, LAST_ENEMY_KEY, Route::Enemies { segments: Vec::new() });
        }
    }));

    use_hook(move || {
        let config = settings.read().scanner_config(None);

        catalogs.load_enemies(&platform, config);
    });

    let globals = context::globals();
    let load = catalogs.enemies();
    let class = if selected.is_some() { "browser showing-detail" } else { "browser" };

    if let Some(fault) = library.fault() {
        return rsx! { Status { title: "The file host could not be reached", detail: fault.to_string() } };
    }

    let (enemies, scanning) = match &load {
        Load::Ready(enemies) => (Some(Rc::clone(enemies)), None),
        Load::Loading(progress) => (None, Some(*progress)),
        Load::Idle => (None, Some(crate::platform::Progress { done: 0, total: 0 })),
        Load::Failed(fault) => {
            return rsx! { Status { title: "Enemies could not be loaded", detail: fault.to_string() } };
        }
    };
    let Some(globals) = globals else {
        let phase = library.phase();

        return rsx! { Status { title: phase.label.to_string(), detail: Library::detail(phase.done, phase.total) } };
    };
    let Some(enemies) = enemies else {
        let detail = scanning.and_then(|progress| Library::detail(progress.done, progress.total));
        let title = if scanning.is_some_and(|progress| progress.total > 0) { "Scanning Enemies..." } else { "Loading Enemies..." };

        return rsx! { Status { title, detail } };
    };
    let enemy = selected.and_then(|id| enemies.iter().find(|enemy| enemy.id == id).cloned());

    if let Some(enemy) = &enemy {
        state.select(enemy.id);
    }

    rsx! {
        Settled {
            div { class: "{class}",
                list::Roster { enemies: list::Entries(Rc::clone(&enemies)), selected }
                div { class: "browser-main",
                    match enemy {
                        Some(enemy) => rsx! { Detail { enemy: SelectedEnemy(Rc::new(enemy)), globals: globals.clone() } },
                        None => rsx! { div { class: "pick-prompt", "Select a Unit" } },
                    }
                }
            }
        }
        FilterPopup { enemies: list::Entries(enemies), globals }
    }
}

#[component]
fn Detail(enemy: SelectedEnemy, globals: Shared) -> Element {
    let settings = context::settings();
    let mut state = state();
    let pictures = images::pictures();
    let navigator = use_navigator();
    let enemy = enemy.0;
    let id = enemy.id;
    let show_invalid = settings.read().show_invalid_enemies();
    let shared = globals.clone();
    let stats = use_memo(use_reactive!(|(id, shared, show_invalid)| scanner::scan_single(id, shared.0.vault(), show_invalid).map(|entry| entry.stats)));
    let warmer = use_context::<Platform>();
    let warm_for = Rc::clone(&enemy);
    let warm_with = globals.clone();

    use_effect(use_reactive!(|id| {
        let _ = id;

        if let Some(files) = enemy_animation::rig_files(&warm_for, &warm_with.0.vault().vfs) {
            spawn(warmer.host().warm(files.names().into_iter().map(Box::from).collect()));
        }
    }));
    let tab = (state.tab)();
    let tab_buttons = TABS.iter().map(|(which, label)| {
        let which = *which;
        let class = header_class(tab == which, true, false);

        rsx! {
            button { key: "{label}", class: "{class}", onclick: move |_| state.tab.set(which), "{label}" }
        }
    });
    let icon = icon_url(&pictures, &globals, &enemy);
    let name = enemy.display_name();
    let mag_input = (state.mag_input)();

    rsx! {
        Hold { stamp: format!("{id}:{tab:?}"),
        div { class: "detail",
            div { class: "detail-header",
                div { class: "header-buttons",
                    div { class: "tab-row", {tab_buttons} }
                }
                div { class: "rule" }
                div { class: "detail-row",
                    div { class: "icon-box bottom",
                        if let Some((url, width, height)) = icon {
                            Pic { url, alt: name.clone(), style: format!("width: {width}px; height: {height}px;") }
                        }
                    }
                    div { class: "info-box",
                        NameBox { name: name.clone(), width: 123.0, height: 56.0, wrap_width: 145.0 }
                        div { class: "id-text weak", "ID: {enemy.id:03}-E" }
                        label { class: "level-row",
                            span { "Magnify:" }
                            input {
                                class: "input level-input",
                                r#type: "text",
                                inputmode: "numeric",
                                placeholder: "100",
                                value: "{mag_input}",
                                oninput: move |event| state.set_magnification(event.value()),
                            }
                            span { "%" }
                        }
                    }
                    if tab == DetailTab::Abilities {
                        div { class: "vrule tall" }
                        export::ExportButtons { request: statblock_request(Rc::clone(&enemy), globals.clone(), settings, pictures.clone(), state) }
                    }
                    if tab == DetailTab::Animation {
                        div { class: "vrule tall" }
                        crate::systems::animation::DebugToggles {}
                    }
                    if tab == DetailTab::Details {
                        div { class: "vrule tall" }
                        button {
                            class: "btn btn-primary appearances-btn",
                            onclick: move |_| {
                                state.appearances.set(Some(id));
                                navigator.push(Route::Stages { segments: Vec::new() });
                            },
                            "Appearances"
                        }
                    }
                }
                div { class: "rule" }
            }
            div { class: if tab == DetailTab::Animation { "detail-body viewing" } else { "detail-body" },
                match tab {
                    DetailTab::Abilities => {
                        let held = stats();
                        let current = held.as_ref().unwrap_or(&enemy.stats);

                        abilities_tab(&enemy, current, &globals, &pictures, (state.magnification)())
                    }
                    DetailTab::Details => details_tab(&enemy),
                    DetailTab::Animation => rsx! { crate::systems::animation::EnemyViewer { enemy: SelectedEnemy(Rc::clone(&enemy)), globals: globals.clone() } },
                }
            }
        }
        }
    }
}

fn statblock_request(enemy: Rc<EnemyEntry>, globals: Shared, settings: Signal<Settings>, pictures: Pictures, state: EnemyState) -> Callback<(), Option<Statblock>> {
    Callback::new(move |()| {
        let data = enemy_statblock::build(Subject {
            enemy: &enemy,
            magnification: *state.magnification.peek(),
            show_invalid: settings.peek().show_invalid_enemies(),
            global: globals.0.context(),
            vault: globals.0.vault(),
        });

        Some(Statblock { data, sheets: icons::sheets(&pictures, &globals, IMG015), priority: settings.peek().general.language_priority.clone() })
    })
}

fn icon_url(pictures: &Pictures, globals: &Shared, enemy: &EnemyEntry) -> Option<(Rc<str>, f32, f32)> {
    let vfs = &globals.0.vault().vfs;
    let sized = enemy
        .icon_path
        .as_ref()
        .and_then(|path| pictures.trimmed(path))
        .or_else(|| vfs.find(EMPTY_ICON).and_then(|fallback| pictures.trimmed(&fallback)))?;
    let scale = (ICON_BOX_WIDTH / sized.width as f32).min(ICON_BOX_HEIGHT / sized.height as f32);

    Some((sized.url, sized.width as f32 * scale, sized.height as f32 * scale))
}

fn abilities_tab(enemy: &EnemyEntry, stats: &Entity, globals: &Shared, pictures: &Pictures, magnification: Magnification) -> Element {
    let ctx = RenderContext::enemy(globals.0.context(), stats, magnification);
    let (traits, headline_1, headline_2, body_1, body_2, footer) = collect_ability_data(&ctx);
    let stat_ctx = StatContext::enemy(stats, enemy.atk_anim_frames, magnification, &globals.0.treasure);
    let cycle = (STAT_ATK_CYCLE.get_value)(&stat_ctx);
    let _ = Faction::Enemy;

    rsx! {
        div { class: "abilities",
            div { class: "stat-grid",
                div { class: "stat-row",
                    StatHeader { label: STAT_ATTACK.display_name }
                    StatHeader { label: STAT_DPS.display_name }
                    StatHeader { label: STAT_RANGE.display_name }
                    StatHeader { label: STAT_ATK_CYCLE.display_name }
                }
                div { class: "stat-row",
                    StatValue { header: STAT_ATTACK.display_name, value: format_stat(&STAT_ATTACK, &stat_ctx) }
                    StatValue { header: STAT_DPS.display_name, value: format_stat(&STAT_DPS, &stat_ctx) }
                    StatValue { header: STAT_RANGE.display_name, value: format_stat(&STAT_RANGE, &stat_ctx) }
                    StatFrames { header: STAT_ATK_CYCLE.display_name, frames: cycle }
                }
                div { class: "stat-row",
                    StatHeader { label: STAT_HITPOINTS.display_name }
                    StatHeader { label: STAT_KNOCKBACKS.display_name }
                    StatHeader { label: STAT_SPEED.display_name }
                    StatHeader { label: STAT_CASH_DROP.display_name }
                }
                div { class: "stat-row",
                    StatValue { header: STAT_HITPOINTS.display_name, value: format_stat(&STAT_HITPOINTS, &stat_ctx) }
                    StatValue { header: STAT_KNOCKBACKS.display_name, value: format_stat(&STAT_KNOCKBACKS, &stat_ctx) }
                    StatValue { header: STAT_SPEED.display_name, value: format_stat(&STAT_SPEED, &stat_ctx) }
                    StatValue { header: STAT_CASH_DROP.display_name, value: format_stat(&STAT_CASH_DROP, &stat_ctx) }
                }
            }
            div { class: "ability-groups",
                if !traits.is_empty() {
                    {abilities::icon_row(pictures, &traits)}
                }
                if !headline_1.is_empty() {
                    {abilities::icon_row(pictures, &headline_1)}
                }
                if !headline_2.is_empty() {
                    {abilities::icon_row(pictures, &headline_2)}
                }
                for items in [&body_1, &body_2] {
                    if !items.is_empty() {
                        div { class: "ability-list",
                            for item in items.iter() {
                                {abilities::ability_row(pictures, item)}
                            }
                        }
                    }
                }
                if !footer.is_empty() {
                    {abilities::icon_row(pictures, &footer)}
                }
            }
        }
    }
}

fn details_tab(enemy: &EnemyEntry) -> Element {
    let description = Some(enemy.description.join("\n")).filter(|text| !text.trim().is_empty()).unwrap_or_else(|| "No description available".to_owned());

    rsx! {
        div { class: "details",
            div { class: "details-section",
                div { class: "details-heading", "Description" }
                pre { class: "details-text", "{description}" }
            }
        }
    }
}
