pub(crate) mod abilities;
mod details;
mod filter;
pub(crate) mod icons;
pub(crate) mod list;
mod talents;

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use dioxus::prelude::*;
use dioxus::router::Navigator;

use codex::combat::Entity;
use kokoro::domains::cat::filter::CatFilterState;
use kokoro::domains::cat::game::stats::seeded_level;
use kokoro::domains::cat::animation as cat_animation;
use kokoro::domains::cat::scanner::CatEntry;
use kokoro::domains::cat::statblock::{self as cat_statblock, Subject};
use kokoro::domains::cat::waiter::unitid;
use kokoro::domains::settings::Settings;

use crate::app::Route;
use crate::platform::Platform;
use crate::systems::catalog::{self, Load};
use crate::systems::context::{self, Shared};
use crate::systems::export::{self, Statblock};
use crate::systems::images::{self, Pictures};
use crate::systems::library::Library;
use crate::systems::picture::Pic;
use crate::widget::{Hold, NameBox, Settled, Status};

pub(crate) use filter::FilterPopup;

const FORM_LABELS: [&str; 4] = ["Normal", "Evolved", "True", "Ultra"];
const EMPTY_CAT_ICON: &str = "uni.png";
const TALENT_HISTORY_CAP: usize = 3;
const ULTRA_BUMP: i32 = 60;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum DetailTab {
    #[default]
    Abilities,
    Talents,
    Details,
    Animation,
}

const TABS: [(DetailTab, &str); 4] =
    [(DetailTab::Abilities, "Abilities"), (DetailTab::Talents, "Talents"), (DetailTab::Details, "Details"), (DetailTab::Animation, "Animation")];

#[derive(Clone, Copy)]
pub(crate) struct CatState {
    pub(crate) search: Signal<String>,
    pub(crate) filter: Signal<CatFilterState>,
    pub(crate) filter_open: Signal<bool>,
    pub(crate) form: Signal<usize>,
    pub(crate) tab: Signal<DetailTab>,
    pub(crate) level_input: Signal<String>,
    pub(crate) level: Signal<i32>,
    pub(crate) talent_levels: Signal<HashMap<u32, HashMap<u8, u8>>>,
    pub(crate) talent_inputs: Signal<HashMap<u8, String>>,
    pub(crate) talent_history: Signal<Vec<u32>>,
    pub(crate) expanded_groups: Signal<HashMap<(u32, u8), bool>>,
    pub(crate) conjure_expanded: Signal<HashMap<u32, bool>>,
    pub(crate) seeded_for: Signal<Option<u32>>,
    ultra_active: Signal<bool>,
    saved_level: Signal<Option<(i32, String)>>,
}

pub(crate) fn provide() -> CatState {
    use_context_provider(|| CatState {
        search: Signal::new(String::new()),
        filter: Signal::new(CatFilterState::default()),
        filter_open: Signal::new(false),
        form: Signal::new(0),
        tab: Signal::new(DetailTab::Abilities),
        level_input: Signal::new(String::from("1")),
        level: Signal::new(1),
        talent_levels: Signal::new(HashMap::new()),
        talent_inputs: Signal::new(HashMap::new()),
        talent_history: Signal::new(Vec::new()),
        expanded_groups: Signal::new(HashMap::new()),
        conjure_expanded: Signal::new(HashMap::new()),
        seeded_for: Signal::new(None),
        ultra_active: Signal::new(false),
        saved_level: Signal::new(None),
    })
}

pub(crate) fn state() -> CatState {
    use_context::<CatState>()
}

fn typable_level(value: &str) -> bool {
    value.chars().all(|glyph| glyph.is_ascii_digit() || glyph == '+')
}

fn parsed_level(input: &str) -> i32 {
    input.split('+').filter_map(|term| term.trim().parse::<i32>().ok()).sum::<i32>().max(1)
}

impl CatState {
    fn clamped(&self, cat: &CatEntry) -> (usize, DetailTab) {
        let max_form = cat.forms.iter().rposition(|&exists| exists).unwrap_or(0);
        let current = (self.form)();
        let form = if current > max_form || !cat.forms.get(current).copied().unwrap_or(false) { max_form } else { current };
        let tab = (self.tab)();
        let tab = if tab == DetailTab::Talents && (form < 2 || cat.talent_data.is_none()) { DetailTab::Abilities } else { tab };

        (form, tab)
    }

    fn select(&mut self, cat: &CatEntry, settings: &Settings) {
        if (self.seeded_for)() == Some(cat.id) {
            return;
        }

        self.seeded_for.set(Some(cat.id));
        self.talent_inputs.write().clear();

        {
            let mut history = self.talent_history.write();

            history.retain(|held| *held != cat.id);
            history.push(cat.id);

            while history.len() > TALENT_HISTORY_CAP {
                let evicted = history.remove(0);

                self.talent_levels.write().remove(&evicted);
            }
        }

        let (level, input) = seeded_level(cat, settings);

        self.level.set(level);
        self.level_input.set(input);

        let (form, tab) = self.clamped(cat);

        self.form.set(form);
        self.tab.set(tab);
        self.ultra_active.set(false);
        self.saved_level.set(None);
        self.sync_ultra(cat, settings);
    }

    pub(crate) fn seed(&mut self, cat: &CatEntry, form: usize, level: &str, talents: &HashMap<u8, u8>) {
        self.seeded_for.set(Some(cat.id));
        self.talent_inputs.write().clear();
        self.talent_levels.write().insert(cat.id, talents.clone());
        self.level.set(parsed_level(level));
        self.level_input.set(level.to_owned());
        self.form.set(form.min(cat.forms.len().saturating_sub(1)));
        self.tab.set(DetailTab::Abilities);
        self.ultra_active.set(false);
        self.saved_level.set(None);
    }

    pub(crate) fn set_form(&mut self, cat: &CatEntry, form: usize, settings: &Settings) {
        self.form.set(form);

        if (self.tab)() == DetailTab::Talents && form < 2 {
            self.tab.set(DetailTab::Abilities);
        }

        self.sync_ultra(cat, settings);
    }

    pub(crate) fn set_level_input(&mut self, input: String) {
        if !typable_level(&input) {
            return;
        }

        self.level.set(parsed_level(&input));
        self.level_input.set(input);
    }

    pub(crate) fn levels_for(&self, id: u32) -> HashMap<u8, u8> {
        self.talent_levels.read().get(&id).cloned().unwrap_or_default()
    }

    pub(crate) fn set_talent_level(&mut self, cat: &CatEntry, index: u8, level: u8, settings: &Settings) {
        self.talent_levels.write().entry(cat.id).or_default().insert(index, level);
        self.talent_inputs.write().remove(&index);
        self.sync_ultra(cat, settings);
    }

    pub(crate) fn set_talent_input(&mut self, cat: &CatEntry, index: u8, input: String, settings: &Settings) {
        if !input.chars().all(|glyph| glyph.is_ascii_digit()) {
            return;
        }

        let max_level = cat.talent_data.as_ref().and_then(|data| data.groups.get(usize::from(index))).map_or(u8::MAX, |group| group.max_level.max(1));
        let trimmed = input.trim();
        let parsed = trimmed.parse::<u8>().ok();

        {
            let mut all = self.talent_levels.write();
            let levels = all.entry(cat.id).or_default();

            if let Some(parsed) = parsed {
                levels.insert(index, parsed.min(max_level));
            } else if trimmed.is_empty() {
                levels.insert(index, 0);
            }
        }

        let clamped = match parsed {
            Some(parsed) if parsed > max_level => max_level.to_string(),
            _ => input,
        };

        self.talent_inputs.write().insert(index, clamped);
        self.sync_ultra(cat, settings);
    }

    pub(crate) fn toggle_talents(&mut self, cat: &CatEntry, ultra: bool, settings: &Settings) {
        let Some(talent_data) = cat.talent_data.as_ref() else {
            return;
        };
        let levels = self.levels_for(cat.id);
        let availability = talents::toggle_availability(talent_data, Some(&levels));
        let has_enabled = if ultra { availability.has_ultra_enabled } else { availability.has_normal_enabled };

        {
            let mut all = self.talent_levels.write();
            let levels = all.entry(cat.id).or_default();
            let mut inputs = self.talent_inputs.write();

            for (index, group) in talent_data.groups.iter().enumerate() {
                let targeted = if ultra { group.limit == 1 } else { group.limit != 1 };

                if targeted {
                    let index = index as u8;

                    levels.insert(index, if has_enabled { 0 } else { group.max_level.max(1) });
                    inputs.remove(&index);
                }
            }
        }

        self.sync_ultra(cat, settings);
    }

    fn sync_ultra(&mut self, cat: &CatEntry, settings: &Settings) {
        let form = (self.form)();
        let levels = self.levels_for(cat.id);
        let mut ultra = form == 3;

        if form >= 2 && !ultra {
            ultra = match &cat.talent_data {
                Some(data) => data.groups.iter().enumerate().any(|(index, group)| group.limit == 1 && levels.get(&(index as u8)).is_some_and(|held| *held > 0)),
                None => levels.iter().any(|(index, held)| *index >= 5 && *held > 0),
            };
        }

        if !settings.cat_data.bump_ultra_60 {
            self.ultra_active.set(ultra);
            self.saved_level.set(None);
            return;
        }

        let active = (self.ultra_active)();

        if !active && ultra {
            self.saved_level.set(Some(((self.level)(), (self.level_input)())));

            if (self.level)() < ULTRA_BUMP {
                self.level.set(ULTRA_BUMP);
                self.level_input.set(ULTRA_BUMP.to_string());
            }
        } else if active
            && !ultra
            && let Some((saved_level, saved_input)) = self.saved_level.take()
        {
            let expected = if saved_level < ULTRA_BUMP { ULTRA_BUMP } else { saved_level };

            if (self.level)() == expected {
                self.level.set(saved_level);
                self.level_input.set(saved_input);
            }
        }

        self.ultra_active.set(ultra);
    }
}

pub(crate) struct Selected {
    pub(crate) cat: CatEntry,
    pub(crate) stats: Option<Arc<Vec<Entity>>>,
}

#[component]
pub(crate) fn Cats(segments: Vec<String>) -> Element {
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

    use_hook(move || {
        let config = settings.read().scanner_config(None);

        catalogs.load_cats(&platform, config);
    });

    let globals = context::globals();
    let load = catalogs.cats();
    let wide_class = if selected.is_some() { "browser showing-detail" } else { "browser" };

    if let Some(fault) = library.fault() {
        return rsx! { Status { title: "The file host could not be reached", detail: fault.to_string() } };
    }

    let (cats, scanning) = match &load {
        Load::Ready(cats) => (Some(Rc::clone(cats)), None),
        Load::Loading(progress) => (None, Some(*progress)),
        Load::Idle => (None, Some(crate::platform::Progress { done: 0, total: 0 })),
        Load::Failed(fault) => {
            return rsx! { Status { title: "Cats could not be loaded", detail: fault.to_string() } };
        }
    };
    let Some(globals) = globals else {
        let phase = library.phase();

        return rsx! { Status { title: phase.label.to_string(), detail: Library::detail(phase.done, phase.total) } };
    };
    let Some(cats) = cats else {
        let detail = scanning.and_then(|progress| Library::detail(progress.done, progress.total));
        let title = if scanning.is_some_and(|progress| progress.total > 0) { "Scanning Cats..." } else { "Loading Cats..." };

        return rsx! { Status { title, detail } };
    };
    let cat = selected.and_then(|id| cats.iter().find(|cat| cat.id == id).cloned());

    if let Some(cat) = &cat {
        state.select(cat, &settings.read());
    }

    rsx! {
        Settled {
            div { class: "{wide_class}",
                list::Roster { cats: list::Entries(Rc::clone(&cats)), selected }
                div { class: "browser-main",
                    match cat {
                        Some(cat) => rsx! { Detail { cat: SelectedCat(Rc::new(cat)), globals: globals.clone() } },
                        None => rsx! { div { class: "pick-prompt", "Select a Unit" } },
                    }
                }
            }
        }
        FilterPopup { cats: list::Entries(cats), globals }
    }
}

fn statblock_request(cat: Rc<CatEntry>, globals: Shared, settings: Signal<Settings>, pictures: Pictures, state: CatState) -> Callback<(), Option<Statblock>> {
    Callback::new(move |()| {
        let form = *state.form.peek();
        let level_input = state.level_input.peek().clone();
        let talent_levels = (form >= 2).then(|| state.levels_for(cat.id));
        let is_conjure_expanded = state.conjure_expanded.peek().get(&cat.id).copied().unwrap_or(settings.peek().cat_data.expand_spirit_details);
        let data = cat_statblock::build(Subject {
            cat: &cat,
            form,
            current_level: *state.level.peek(),
            level_input: &level_input,
            talent_levels: talent_levels.as_ref(),
            is_conjure_expanded,
            global: globals.0.context(),
        })?;

        Some(Statblock { data, sheets: icons::sheets(&pictures, &globals, icons::IMG015), priority: settings.peek().general.language_priority.clone() })
    })
}

#[derive(Clone)]
pub(crate) struct SelectedCat(pub(crate) Rc<CatEntry>);

impl PartialEq for SelectedCat {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[component]
pub(crate) fn Detail(cat: SelectedCat, globals: Shared, #[props(default)] extra: Option<Element>) -> Element {
    let settings = context::settings();
    let mut state = state();
    let pictures = images::pictures();
    let cat = cat.0;
    let id = cat.id;
    let shared = globals.clone();
    let stats = use_memo(use_reactive!(|(id, shared)| unitid(&shared.0.vault().vfs, id as i32).map(Arc::new)));
    let form = (state.form)();
    let tab = (state.tab)();
    let warmer = use_context::<Platform>();
    let warm_for = Rc::clone(&cat);
    let warm_with = globals.clone();

    use_effect(use_reactive!(|(id, form)| {
        let _ = id;

        if let Some(files) = cat_animation::rig_files(&warm_for, form, &warm_with.0.vault().vfs) {
            spawn(warmer.host().warm(files.names().into_iter().map(Box::from).collect()));
        }
    }));

    let form_buttons = FORM_LABELS.iter().enumerate().map(|(index, label)| {
        let exists = cat.forms.get(index).copied().unwrap_or(false);
        let class = header_class(form == index, exists, true);
        let cat = Rc::clone(&cat);

        rsx! {
            button {
                key: "{index}",
                class: "{class}",
                disabled: !exists,
                onclick: move |_| state.set_form(&cat, index, &settings.read()),
                "{label}"
            }
        }
    });
    let tab_buttons = TABS.iter().map(|(which, label)| {
        let which = *which;
        let available = which != DetailTab::Talents || (form >= 2 && cat.talent_data.is_some());
        let class = header_class(tab == which, available, false);

        rsx! {
            button {
                key: "{label}",
                class: "{class}",
                disabled: !available,
                onclick: move |_| state.tab.set(which),
                "{label}"
            }
        }
    });
    let icon = icon_url(&pictures, &globals, &cat, form);
    let name = cat.display_name(form);
    let level_input = (state.level_input)();
    let selected = Rc::new(Selected { cat: (*cat).clone(), stats: stats() });

    rsx! {
        Hold { stamp: format!("{id}:{form}:{tab:?}"),
        div { class: "detail",
            div { class: "detail-header",
                div { class: "header-buttons",
                    div { class: "form-row", {form_buttons} }
                    div { class: "vrule" }
                    div { class: "tab-row", {tab_buttons} }
                }
                div { class: "rule" }
                div { class: "detail-row",
                    div { class: "icon-box",
                        if let Some(url) = icon {
                            Pic { url, alt: name.clone() }
                        }
                    }
                    div { class: "info-box",
                        NameBox { name: name.clone(), width: 123.0, height: 56.0, wrap_width: 145.0 }
                        div { class: "id-text weak", "ID: {cat.id:03}-{form + 1}" }
                        label { class: "level-row",
                            span { "Level:" }
                            input {
                                class: "input level-input",
                                r#type: "text",
                                inputmode: "numeric",
                                value: "{level_input}",
                                oninput: move |event| state.set_level_input(event.value()),
                            }
                        }
                    }
                    if let Some(extra) = extra {
                        div { class: "vrule tall" }
                        {extra}
                    }
                    if tab == DetailTab::Animation {
                        div { class: "vrule tall" }
                        crate::systems::animation::DebugToggles {}
                    }
                    if tab == DetailTab::Abilities {
                        div { class: "vrule tall" }
                        export::ExportButtons { request: statblock_request(Rc::clone(&cat), globals.clone(), settings, pictures.clone(), state) }
                    }
                    if tab == DetailTab::Talents && cat.talent_data.is_some() {
                        div { class: "vrule tall" }
                        talents::Header { cat: SelectedCat(Rc::clone(&cat)), globals: globals.clone() }
                    }
                }
                div { class: "rule" }
            }
            div { class: if tab == DetailTab::Animation { "detail-body viewing" } else { "detail-body" },
                    match tab {
                        DetailTab::Abilities => rsx! { abilities::Abilities { selected: Picked(Rc::clone(&selected)), globals: globals.clone() } },
                        DetailTab::Talents => rsx! { talents::Talents { selected: Picked(Rc::clone(&selected)), globals: globals.clone() } },
                        DetailTab::Details => rsx! { details::Details { cat: SelectedCat(Rc::clone(&cat)), globals: globals.clone() } },
                        DetailTab::Animation => rsx! { crate::systems::animation::CatViewer { cat: SelectedCat(Rc::clone(&cat)), globals: globals.clone() } },
                    }
            }
        }
        }
    }
}

#[derive(Clone)]
pub(crate) struct Picked(pub(crate) Rc<Selected>);

impl PartialEq for Picked {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

pub(crate) fn header_class(selected: bool, available: bool, square: bool) -> String {
    let mut class = String::from("btn btn-toggle btn-header header-btn");

    if square {
        class.push_str(" square");
    }

    if !available {
        class.push_str(" unavailable");
    } else if selected {
        class.push_str(" active");
    }

    class
}

fn icon_url(pictures: &Pictures, globals: &Shared, cat: &CatEntry, form: usize) -> Option<Rc<str>> {
    let vfs = &globals.0.vault().vfs;
    let path = cat.deploy_icon_paths.get(form).and_then(Option::as_ref);

    path.and_then(|path| pictures.trimmed(path))
        .or_else(|| vfs.find(EMPTY_CAT_ICON).and_then(|fallback| pictures.trimmed(&fallback)))
        .map(|sized| sized.url)
}

pub(crate) fn jump(navigator: &Navigator, id: u32, form: usize, mut state: CatState) {
    state.seeded_for.set(None);
    state.form.set(form);
    navigator.push(Route::cat(id));
}
