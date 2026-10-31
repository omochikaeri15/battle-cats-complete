mod battleground;
mod category;
mod filter;
mod fixedlineup;
mod info;
mod materials;
mod treasure;

use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use dioxus::prelude::*;

use codex::chapter::Category;
use kokoro::domains::settings::SidebarBehavior;
use kokoro::domains::stage::filter::StageFilterState;
use kokoro::domains::stage::filter::enemy::EnemyFilter;
use kokoro::domains::stage::fixedlineup as core_fixedlineup;
use kokoro::domains::stage::scanner::StageBundle;
use kokoro::domains::stage::{GlobalMapId, GlobalStageId, StageDataState, navigate};

use crate::app::{self, Route};
use crate::domains::enemy;
use crate::platform::Platform;
use crate::systems::catalog::{self, Load};
use crate::systems::context::{self, Shared};
use crate::systems::library::Library;
use crate::theme::SLIDE_MS;
use crate::widget::{Hold, Status};

pub(crate) use filter::FilterPopup;

const OPEN_ARROW: &str = "\u{25c0}";
const CLOSED_ARROW: &str = "\u{25b6}";
const CATEGORY_COLUMN_WIDTH: f32 = 180.0;
const COLUMN_WIDTH: f32 = 200.0;
const COLUMN_SEPARATOR_WIDTH: f32 = 15.0;
const SIDEBAR_PADDING: f32 = 15.0;
const SIDEBAR_PUSH_GAP: f32 = 10.0;
const SLIDE_SETTLE_MS: u32 = 40;
const LAST_STAGE_KEY: &str = "last-stage";

#[derive(Clone, Copy)]
pub(crate) struct StageState {
    pub(crate) category: Signal<Option<Category>>,
    pub(crate) map: Signal<Option<GlobalMapId>>,
    pub(crate) stage: Signal<Option<GlobalStageId>>,
    pub(crate) crown: Signal<u8>,
    pub(crate) sidebar_open: Signal<bool>,
    pub(crate) sliding: Signal<bool>,
    pub(crate) filter: Signal<StageFilterState>,
    pub(crate) filter_open: Signal<bool>,
    pub(crate) attributes_for: Signal<Option<usize>>,
}

pub(crate) fn provide() -> StageState {
    use_context_provider(|| StageState {
        category: Signal::new(None),
        map: Signal::new(None),
        stage: Signal::new(None),
        crown: Signal::new(0),
        sidebar_open: Signal::new(true),
        sliding: Signal::new(false),
        filter: Signal::new(StageFilterState::default()),
        filter_open: Signal::new(false),
        attributes_for: Signal::new(None),
    })
}

pub(crate) fn state() -> StageState {
    use_context::<StageState>()
}

pub(crate) struct Data {
    pub(crate) state: StageDataState,
}

#[derive(Clone)]
pub(crate) struct Loaded(pub(crate) Rc<Data>);

impl PartialEq for Loaded {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

fn assemble(bundle: &StageBundle, enemies: Option<&[kokoro::domains::enemy::scanner::EnemyEntry]>) -> StageDataState {
    let mut data = StageDataState {
        registry: bundle.registry.clone(),
        enemy_name_registry: bundle.dictionaries.enemy_name_registry.clone(),
        drop_chara_registry: bundle.dictionaries.drop_chara_registry.clone(),
        unit_buy_registry: bundle.dictionaries.unit_buy_registry.clone(),
        cat_name_registry: bundle.dictionaries.cat_name_registry.clone(),
        lock_skip_registry: bundle.dictionaries.lock_skip_registry.clone(),
        scat_cpu_setting: bundle.dictionaries.scat_cpu_setting.clone(),
        ..StageDataState::default()
    };

    if let Some(enemies) = enemies {
        data.sync_enemies(enemies);
    }

    data
}

#[component]
pub(crate) fn Stages(segments: Vec<String>) -> Element {
    let platform = use_context::<Platform>();
    let navigator = use_navigator();
    let catalogs = catalog::catalogs();
    let mut state = state();
    let wanted = Route::stage_of(&segments);
    let bare = segments.is_empty();
    let mut seen: Signal<Option<GlobalStageId>> = use_signal(|| None);
    let known = match catalogs.stages() {
        Load::Ready(bundle) => wanted.as_ref().filter(|id| bundle.registry.stages.contains_key(*id)).cloned(),
        _ => None,
    };

    use_effect(use_reactive!(|(wanted, bare)| {
        let current = (state.stage)();
        let applied = seen();
        let Load::Ready(bundle) = catalogs.stages() else {
            return;
        };
        let exists = |id: &GlobalStageId| bundle.registry.stages.contains_key(id);

        if bare {
            let saved = || app::last_visited(&platform, LAST_STAGE_KEY).and_then(|saved| Route::stage_of(&saved));

            if let Some(route) = current.or_else(saved).filter(exists).and_then(|id| Route::stage(&id)) {
                navigator.replace(route);
            }

            return;
        }

        let Some(wanted) = wanted.filter(exists) else {
            app::forget_visit(&platform, navigator, LAST_STAGE_KEY, Route::Stages { segments: Vec::new() });

            return;
        };

        if applied.as_ref() != Some(&wanted) {
            return;
        }

        match current.filter(|current| *current != wanted).and_then(|current| Route::stage(&current)) {
            Some(moved) => {
                navigator.replace(moved);
            }
            None => {
                if let Some(route) = Route::stage(&wanted) {
                    app::remember_visit(&platform, LAST_STAGE_KEY, &route);
                }
            }
        }
    }));

    if let Some(id) = known
        && seen.peek().as_ref() != Some(&id)
    {
        seen.set(Some(id.clone()));

        if state.stage.peek().as_ref() != Some(&id) {
            state.category.set(Some(id.category.clone()));
            state.map.set(Some(GlobalMapId { category: id.category.clone(), map: id.map }));
            state.stage.set(Some(id));
        }
    }

    rsx! {
        Browser {}
    }
}

#[component]
pub(crate) fn Browser() -> Element {
    let platform = use_context::<Platform>();
    let slider = platform.clone();
    let library = use_context::<Library>();
    let catalogs = catalog::catalogs();
    let settings = context::settings();
    let mut state = state();
    let mut enemy_state = enemy::state();

    use_hook(move || {
        let config = settings.read().scanner_config(None);

        catalogs.load_stages(&platform, config.clone());
        catalogs.load_enemies(&platform, config);
    });

    if let Some(id) = (enemy_state.appearances)() {
        enemy_state.appearances.set(None);
        state.sidebar_open.set(true);
        state.filter.set(StageFilterState { enemies: vec![EnemyFilter { name_or_id: format!("{id:03}"), ..EnemyFilter::default() }], ..StageFilterState::default() });
    }

    let globals = context::globals();
    let load = catalogs.stages();
    let enemies = catalogs.enemies();

    if let Some(fault) = library.fault() {
        return rsx! { Status { title: "The file host could not be reached", detail: fault.to_string() } };
    }

    let (bundle, scanning) = match &load {
        Load::Ready(bundle) => (Some(Rc::clone(bundle)), None),
        Load::Loading(progress) => (None, Some(*progress)),
        Load::Idle => (None, Some(crate::platform::Progress { done: 0, total: 0 })),
        Load::Failed(fault) => {
            return rsx! { Status { title: "Stages could not be loaded", detail: fault.to_string() } };
        }
    };
    let Some(globals) = globals else {
        let phase = library.phase();

        return rsx! { Status { title: phase.label.to_string(), detail: Library::detail(phase.done, phase.total) } };
    };
    let Some(bundle) = bundle else {
        let detail = scanning.and_then(|progress| Library::detail(progress.done, progress.total));
        let title = if scanning.is_some_and(|progress| progress.total > 0) { "Scanning Stages..." } else { "Loading Stages..." };

        return rsx! { Status { title, detail } };
    };
    let enemy_list = match &enemies {
        Load::Ready(list) => Some(Rc::clone(list)),
        _ => None,
    };
    let inputs = Inputs(bundle, enemy_list);
    let data = use_memo(use_reactive!(|inputs| Loaded(Rc::new(Data { state: assemble(&inputs.0, inputs.1.as_deref().map(Vec::as_slice)) }))));
    let data = data();
    let stage_id = (state.stage)();
    let stage = stage_id.as_ref().and_then(|id| data.0.state.registry.stages.get(id));

    if let Some(stage) = stage
        && (state.crown)() >= stage.max_crowns.max(1)
    {
        state.crown.set(0);
    }

    let open = (state.sidebar_open)();
    let arrow = if open { OPEN_ARROW } else { CLOSED_ARROW };
    let registry = &data.0.state.registry;
    let mut columns = 1usize;

    if (state.category)().is_some() {
        columns += 1;
    }

    if (state.map)().as_ref().is_some_and(|map_id| registry.maps.contains_key(map_id)) {
        columns += 1;
    }

    let span = CATEGORY_COLUMN_WIDTH + (columns - 1) as f32 * (COLUMN_WIDTH + COLUMN_SEPARATOR_WIDTH) + SIDEBAR_PADDING * 2.0;
    let push = settings.read().stages.sidebar_behavior == SidebarBehavior::Push;
    let shift = if push && open { span + SIDEBAR_PUSH_GAP } else { 0.0 };
    let stamp = stage_id.as_ref().map(|id| format!("{}:{}:{}:{}", category::display_name(&id.category), id.map, id.stage, (state.crown)()));
    let sliding = (state.sliding)();
    let main_class = if sliding { "stage-main scroll sliding" } else { "stage-main scroll" };
    let slot_class = match (open, sliding) {
        (true, true) => "stage-sidebar-slot sliding",
        (true, false) => "stage-sidebar-slot",
        (false, true) => "stage-sidebar-slot sliding closed",
        (false, false) => "stage-sidebar-slot closed",
    };
    rsx! {
        div { class: "stage-page",
            div { class: "{main_class}", style: "left: {shift}px;",
                Hold { stamp: stamp.clone().unwrap_or_default(),
                    match stamp {
                        Some(_) => rsx! { StageView { data: data.clone(), globals: globals.clone(), stage_id: stage_id.clone(), crown: (state.crown)() } },
                        None => {
                            let label = if registry.stages.is_empty() { "No Stage Data" } else { "Select a stage to view details" };

                            rsx! { div { class: "pick-prompt small", "{label}" } }
                        }
                    }
                }
            }
            div { class: "stage-sidebar-layer",
                div { class: "{slot_class}", style: "width: {span}px;",
                    div { class: "stage-sidebar", style: "width: {span}px;",
                        Sidebar { data: data.clone(), globals: globals.clone() }
                    }
                }
                button {
                    class: "btn btn-neutral stage-toggle symbols",
                    onclick: move |_| {
                        state.sidebar_open.toggle();
                        state.sliding.set(true);

                        let host = slider.clone();

                        spawn(async move {
                            host.host().sleep(SLIDE_MS + SLIDE_SETTLE_MS).await;
                            state.sliding.set(false);
                        });
                    },
                    "{arrow}"
                }
            }
        }
        FilterPopup { data, globals }
    }
}

#[component]
fn Sidebar(data: Loaded, globals: Shared) -> Element {
    let mut state = state();
    let filter = state.filter.read().clone();
    let fingerprint = digest(&filter);
    let held = data.clone();
    let shared = globals.clone();
    let compiled = use_memo(use_reactive!(|(fingerprint, held, shared)| {
        let _ = fingerprint;
        let compiled = state.filter.read().compile();
        let mut matching_stages = HashSet::new();
        let mut matching_maps = HashSet::new();
        let mut matching_categories = HashSet::new();

        if compiled.is_active() {
            let lookup = kokoro::domains::stage::filter::StageLookupContext::from_data(&held.0.state, shared.0.vault(), &shared.0.treasure);

            for (key, stage) in &held.0.state.registry.stages {
                let map_key = GlobalMapId { category: key.category.clone(), map: key.map };
                let Some(map) = held.0.state.registry.maps.get(&map_key) else {
                    continue;
                };

                if compiled.matches(category::display_name(&key.category), map, stage, &lookup) {
                    matching_stages.insert(key.clone());
                    matching_maps.insert(map_key);
                    matching_categories.insert(key.category.clone());
                }
            }
        }

        Matching { active: compiled.is_active(), stages: matching_stages, maps: matching_maps, categories: matching_categories }
    }));
    let matching = compiled.read();
    let registry = &data.0.state.registry;
    let mut categories = navigate::get_categories(registry);

    categories.sort_by_key(category::sort_order);

    let selected_category = (state.category)();
    let selected_map = (state.map)();
    let selected_stage = (state.stage)();
    let filter_class = if filter.is_active() { "btn btn-toggle active stage-filter-btn" } else { "btn btn-toggle stage-filter-btn" };
    let shown_categories: Vec<&Category> = categories.iter().filter(|category| !matching.active || matching.categories.contains(category)).collect();

    let stamp = format!(
        "{}:{}:{fingerprint}",
        selected_category.as_ref().map(category::display_name).unwrap_or_default(),
        selected_map.as_ref().map_or(0, |map_id| map_id.map)
    );

    rsx! {
        button { class: "{filter_class}", onclick: move |_| state.filter_open.set(true), "Filter" }
        div { class: "rule" }
        Hold { stamp,
            div { class: "stage-columns",
                div { class: "stage-column categories scroll",
                    if shown_categories.is_empty() {
                        div { class: "column-empty", "No Categories Found!" }
                    }
                    for category in shown_categories {
                        {
                            let picked = category.clone();
                            let selected = selected_category.as_ref() == Some(category);

                            rsx! {
                                button {
                                    class: if selected { "list-row filled selected stage-row" } else { "list-row filled stage-row" },
                                    onclick: move |_| {
                                        state.category.set(Some(picked.clone()));
                                        state.map.set(None);
                                        state.stage.set(None);
                                    },
                                    "{category::display_name(category)}"
                                }
                            }
                        }
                    }
                }
                if let Some(category) = selected_category.as_ref() {
                    div { class: "vrule full" }
                    div { class: "stage-column scroll",
                        {
                            let maps: Vec<_> = navigate::get_maps(registry, category)
                                .into_iter()
                                .filter(|map| !matching.active || matching.maps.contains(&GlobalMapId { category: category.clone(), map: map.map_id }))
                                .collect();

                            rsx! {
                                if maps.is_empty() {
                                    div { class: "column-empty", "No Maps Found!" }
                                }
                                for map in maps {
                                    {
                                        let key = GlobalMapId { category: category.clone(), map: map.map_id };
                                        let selected = selected_map.as_ref() == Some(&key);

                                        rsx! {
                                            button {
                                                key: "{map.map_id}",
                                                class: if selected { "list-row filled selected stage-row" } else { "list-row filled stage-row" },
                                                onclick: move |_| {
                                                    state.map.set(Some(key.clone()));
                                                    state.stage.set(None);
                                                },
                                                "{map.name}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(map_id) = selected_map.as_ref().filter(|map_id| registry.maps.contains_key(map_id)) {
                    div { class: "vrule full" }
                    div { class: "stage-column scroll",
                        {
                            let stages: Vec<_> = navigate::get_stages(registry, map_id)
                                .into_iter()
                                .filter(|stage| !matching.active || matching.stages.contains(&GlobalStageId { category: map_id.category.clone(), map: map_id.map, stage: stage.stage_id }))
                                .collect();

                            rsx! {
                                if stages.is_empty() {
                                    div { class: "column-empty", "No Stages Found!" }
                                }
                                for stage in stages {
                                    {
                                        let key = GlobalStageId { category: map_id.category.clone(), map: map_id.map, stage: stage.stage_id };
                                        let selected = selected_stage.as_ref() == Some(&key);

                                        rsx! {
                                            button {
                                                key: "{stage.stage_id}",
                                                class: if selected { "list-row filled selected stage-row" } else { "list-row filled stage-row" },
                                                onclick: move |_| state.stage.set(Some(key.clone())),
                                                "{stage.name}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
struct Inputs(Rc<StageBundle>, Option<Rc<Vec<kokoro::domains::enemy::scanner::EnemyEntry>>>);

impl PartialEq for Inputs {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
            && match (&self.1, &other.1) {
                (Some(left), Some(right)) => Rc::ptr_eq(left, right),
                (None, None) => true,
                _ => false,
            }
    }
}

fn digest(filter: &StageFilterState) -> u64 {
    let mut hasher = DefaultHasher::new();

    filter.hash(&mut hasher);
    hasher.finish()
}

#[derive(PartialEq)]
struct Matching {
    active: bool,
    stages: HashSet<GlobalStageId>,
    maps: HashSet<GlobalMapId>,
    categories: HashSet<Category>,
}

#[component]
fn StageView(data: Loaded, globals: Shared, stage_id: Option<GlobalStageId>, crown: u8) -> Element {
    let Some(stage) = stage_id.as_ref().and_then(|stage_id| data.0.state.registry.stages.get(stage_id)) else {
        return rsx! {};
    };
    let map_key = GlobalMapId { category: stage.category.clone(), map: stage.map_id };
    let Some(map) = data.0.state.registry.maps.get(&map_key) else {
        return rsx! {};
    };
    let vfs = &globals.0.vault().vfs;
    let preset = stage.fixed_lineups.get(&crown);
    let lineup = preset.map(|preset| core_fixedlineup::resolve_lineup(vfs, preset));

    rsx! {
        div { class: "stage-content",
            {info::view(stage, map, &data.0.state, &globals, crown)}
            if materials::has_drops(map) {
                div { class: "drop-tables",
                    {materials::view(stage, map, crown, &globals)}
                    {treasure::view(stage, &data.0.state, &globals)}
                }
            } else {
                {treasure::view(stage, &data.0.state, &globals)}
            }
            if let (Some(preset), Some(lineup)) = (preset, lineup.as_ref()) {
                {fixedlineup::view(lineup, preset, &globals)}
            }
            {battleground::view(stage, map, crown, &data.0.state, &globals)}
        }
    }
}
