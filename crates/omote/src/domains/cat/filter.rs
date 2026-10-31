use std::collections::HashSet;
use std::rc::Rc;

use dioxus::prelude::*;

use codex::combat::{AttrUnit, Identity, REGISTRY};
use kokoro::domains::cat::combo::{self, ComboEffects};
use kokoro::domains::cat::filter::{ATTACK_TYPE_ICONS, CatFilterState, MatchMode, TalentFilterMode, icons as filter_icons};
use kokoro::systems::combat::registry::{AbilityIcon, DisplayGroup, get_display_def};
use kokoro::systems::combat::{CustomIcon, TalentIdentities, present_identities, talent_identities};

use crate::systems::context::Shared;
use crate::systems::images;
use crate::systems::library::Library;
use crate::widget::{Popup, RangeRow, Tip};

use super::icons::{self, ICON_SIZE};
use super::list::Entries;
use super::state;

const STAT_KEYS: [&str; 9] = ["Attack", "Dps", "Range", "Atk Cycle (f)", "Hitpoints", "Knockbacks", "Speed", "Cooldown (f)", "Cost"];
const RARITY_LABELS: [&str; 6] = ["Normal", "Special", "Rare", "Super Rare", "Uber Rare", "Legend Rare"];
const FORM_FILTER_LABELS: [&str; 4] = ["Normal Form", "Evolved Form", "True Form", "Ultra Form"];
const TALENT_MODES: [TalentFilterMode; 3] = [TalentFilterMode::Ignore, TalentFilterMode::Consider, TalentFilterMode::Only];

struct Available {
    inherent: HashSet<Identity>,
    gainable: TalentIdentities,
    combos: ComboEffects,
}

#[derive(Clone)]
struct Found(Rc<Available>);

impl PartialEq for Found {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

fn talent_mode(label: &str) -> TalentFilterMode {
    TALENT_MODES.into_iter().find(|mode| mode.label() == label).unwrap_or_default()
}

fn ability_schema(icon: AbilityIcon) -> &'static [(&'static str, AttrUnit)] {
    REGISTRY.iter().find(|def| get_display_def(def.identity).icon == icon).map_or(&[], |def| def.schema)
}

fn toggle_icon(mut state: super::CatState, icon: AbilityIcon) {
    let mut held = state.filter.write();

    if !held.active_icons.remove(&icon) {
        held.active_icons.insert(icon);
    }
}

fn gainable(filter: &CatFilterState, available: &Available, identity: Identity) -> bool {
    let normal = filter.talent_mode != TalentFilterMode::Ignore && available.gainable.normal.contains(&identity);
    let ultra = filter.ultra_talent_mode != TalentFilterMode::Ignore && available.gainable.ultra.contains(&identity);

    normal || ultra
}

fn group_icons(available: &Available, target: DisplayGroup, rendered: &mut HashSet<AbilityIcon>) -> Vec<AbilityIcon> {
    let mut found = Vec::new();

    for def in REGISTRY.iter() {
        if !available.inherent.contains(&def.identity) {
            continue;
        }

        let display = get_display_def(def.identity);

        if display.group != target || display.group == DisplayGroup::Trait || ATTACK_TYPE_ICONS.contains(&display.icon) || found.contains(&display.icon) {
            continue;
        }

        found.push(display.icon);
        rendered.insert(display.icon);
    }

    if target == DisplayGroup::Headline2 {
        let kamikaze = AbilityIcon::Custom(CustomIcon::Kamikaze);

        if !found.contains(&kamikaze) {
            found.push(kamikaze);
            rendered.insert(kamikaze);
        }
    }

    found
}

#[component]
pub(crate) fn FilterPopup(cats: Entries, globals: Shared) -> Element {
    let library = use_context::<Library>();
    let state = state();
    let generation = library.generation();
    let shared = globals.clone();
    let available = use_memo(use_reactive!(|(cats, shared, generation)| {
        let _ = generation;

        Found(Rc::new(Available {
            inherent: present_identities(cats.0.iter().flat_map(|cat| cat.stats.iter().flatten())),
            gainable: talent_identities(cats.0.iter().filter_map(|cat| cat.talent_data.as_ref())),
            combos: combo::effects(shared.0.vault()),
        }))
    }));

    rsx! {
        Popup { title: "Advanced Cat Filter", open: state.filter_open,
            Content { available: available(), globals }
        }
    }
}

#[component]
fn Content(available: Found, globals: Shared) -> Element {
    let mut state = state();
    let pictures = images::pictures();
    let filter = state.filter.read().clone();
    let available = available.0;
    let traits: Vec<AbilityIcon> = REGISTRY
        .iter()
        .filter(|def| available.inherent.contains(&def.identity))
        .map(|def| get_display_def(def.identity))
        .filter(|display| display.group == DisplayGroup::Trait)
        .map(|display| display.icon)
        .collect();
    let mut rendered: HashSet<AbilityIcon> = HashSet::new();
    let headline_1 = group_icons(&available, DisplayGroup::Headline1, &mut rendered);
    let headline_2 = group_icons(&available, DisplayGroup::Headline2, &mut rendered);
    let body_1 = group_icons(&available, DisplayGroup::Body1, &mut rendered);
    let body_2 = group_icons(&available, DisplayGroup::Body2, &mut rendered);
    let footer = group_icons(&available, DisplayGroup::Footer, &mut rendered);
    let check_talents = filter.talent_mode != TalentFilterMode::Ignore || filter.ultra_talent_mode != TalentFilterMode::Ignore;
    let mut talents: Vec<AbilityIcon> = Vec::new();

    if check_talents {
        for def in REGISTRY.iter() {
            if !gainable(&filter, &available, def.identity) {
                continue;
            }

            let display = get_display_def(def.identity);

            if display.group == DisplayGroup::Trait || rendered.contains(&display.icon) || ATTACK_TYPE_ICONS.contains(&display.icon) || talents.contains(&display.icon) {
                continue;
            }

            talents.push(display.icon);
        }
    }

    let icon_wrap = |items: &[AbilityIcon]| {
        rsx! {
            div { class: "icon-row filter-icons",
                for icon in items.iter().copied() {
                    {
                        let active = filter.active_icons.contains(&icon);
                        let name = filter_icons::get_icon_name(&icon, &globals.0.names);

                        rsx! {
                            Tip { text: name,
                                button {
                                    class: if active { "icon-toggle active" } else { "icon-toggle" },
                                    onclick: move |_| toggle_icon(state, icon),
                                    {icons::ability_icon(&pictures, icon, ICON_SIZE, "?")}
                                }
                            }
                        }
                    }
                }
            }
        }
    };
    let labeled = |items: &[AbilityIcon]| {
        rsx! {
            div { class: "filter-labeled",
                for icon in items.iter().copied() {
                    {
                        let active = filter.active_icons.contains(&icon);
                        let name = filter_icons::get_icon_name(&icon, &globals.0.names);
                        let schema = ability_schema(icon);
                        let expanded = active && !schema.is_empty();

                        rsx! {
                            div { class: if expanded { "card filter-ability expanded" } else { "filter-ability" },
                                div { class: "filter-ability-header",
                                    button { class: if active { "icon-toggle active" } else { "icon-toggle" }, onclick: move |_| toggle_icon(state, icon),
                                        {icons::ability_icon(&pictures, icon, ICON_SIZE, "?")}
                                    }
                                    button { class: "btn btn-text", onclick: move |_| toggle_icon(state, icon), "{name}" }
                                }
                                if expanded {
                                    div { class: "filter-ranges",
                                        for (attr, _) in schema.iter().copied() {
                                            {
                                                let range = filter.adv_ranges.get(&icon).and_then(|ranges| ranges.get(attr));
                                                let min = range.map_or(String::new(), |range| range.min.clone());
                                                let max = range.map_or(String::new(), |range| range.max.clone());

                                                rsx! {
                                                    RangeRow {
                                                        label: attr,
                                                        min,
                                                        max,
                                                        on_min: move |value| { state.filter.write().adv_ranges.entry(icon).or_default().entry(attr).or_default().min = value; },
                                                        on_max: move |value| { state.filter.write().adv_ranges.entry(icon).or_default().entry(attr).or_default().max = value; },
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
    };
    let combos = (!available.combos.groups.is_empty()).then(|| {
        rsx! {
            div { class: "filter-section",
                div { class: "filter-heading", "Combo" }
                for group in available.combos.groups.iter() {
                    div { class: "toggle-row",
                        for effect in group.iter().copied() {
                            if let Some(label) = available.combos.labels.get(&effect) {
                                {
                                    let active = filter.combo_effects.contains(&effect);
                                    let units = Rc::clone(&available);

                                    rsx! {
                                        button {
                                            class: if active { "btn btn-toggle active" } else { "btn btn-toggle" },
                                            onclick: move |_| {
                                                let mut held = state.filter.write();

                                                if !held.combo_effects.remove(&effect) {
                                                    held.combo_effects.insert(effect);
                                                }

                                                held.combo_units = held.combo_effects.iter().filter_map(|effect| units.combos.units.get(effect)).flatten().copied().collect();
                                            },
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    rsx! {
        div { class: "filter-content",
            div { class: "filter-section",
                div { class: "filter-heading", "Attributes" }
                div { class: "toggle-row",
                    for (index, label) in RARITY_LABELS.iter().enumerate() {
                        button {
                            class: if filter.rarities[index] { "btn btn-toggle active" } else { "btn btn-toggle" },
                            onclick: move |_| { let mut held = state.filter.write(); held.rarities[index] = !held.rarities[index]; },
                            "{label}"
                        }
                    }
                }
                div { class: "toggle-row",
                    for (index, label) in FORM_FILTER_LABELS.iter().enumerate() {
                        button {
                            class: if filter.forms[index] { "btn btn-toggle active" } else { "btn btn-toggle" },
                            onclick: move |_| { let mut held = state.filter.write(); held.forms[index] = !held.forms[index]; },
                            "{label}"
                        }
                    }
                }
                div { class: "mode-row",
                    label { "Mode:"
                        select { class: "combo", value: if filter.match_mode == MatchMode::And { "And" } else { "Or" },
                            onchange: move |event| state.filter.write().match_mode = if event.value() == "And" { MatchMode::And } else { MatchMode::Or },
                            option { value: "And", selected: filter.match_mode == MatchMode::And, "And" }
                            option { value: "Or", selected: filter.match_mode == MatchMode::Or, "Or" }
                        }
                    }
                    label { "Talents:"
                        select { class: "combo", value: "{filter.talent_mode.label()}",
                            onchange: move |event| state.filter.write().talent_mode = talent_mode(&event.value()),
                            for mode in TALENT_MODES {
                                option { value: "{mode.label()}", selected: filter.talent_mode == mode, "{mode.label()}" }
                            }
                        }
                    }
                    label { "Ultra Talents:"
                        select { class: "combo", value: "{filter.ultra_talent_mode.label()}",
                            onchange: move |event| state.filter.write().ultra_talent_mode = talent_mode(&event.value()),
                            for mode in TALENT_MODES {
                                option { value: "{mode.label()}", selected: filter.ultra_talent_mode == mode, "{mode.label()}" }
                            }
                        }
                    }
                }
            }
            div { class: "filter-section",
                div { class: "filter-heading", "Stats" }
                label { class: "level-filter", "Target Level:"
                    input { class: "input", r#type: "text", inputmode: "numeric", placeholder: "Any", value: "{filter.level_input}",
                        oninput: move |event| state.filter.write().level_input = event.value() }
                }
                div { class: "stat-ranges",
                    for stat in STAT_KEYS {
                        {
                            let range = filter.stat_ranges.get(stat);
                            let min = range.map_or(String::new(), |range| range.min.clone());
                            let max = range.map_or(String::new(), |range| range.max.clone());

                            rsx! {
                                RangeRow {
                                    label: stat,
                                    min,
                                    max,
                                    on_min: move |value| { state.filter.write().stat_ranges.entry(stat).or_default().min = value; },
                                    on_max: move |value| { state.filter.write().stat_ranges.entry(stat).or_default().max = value; },
                                }
                            }
                        }
                    }
                }
            }
            div { class: "filter-section",
                div { class: "filter-heading", "Target Traits" }
                {icon_wrap(&traits)}
            }
            div { class: "filter-section",
                div { class: "filter-heading", "Attack Type" }
                {icon_wrap(ATTACK_TYPE_ICONS)}
            }
            div { class: "filter-section",
                div { class: "filter-heading", "Abilities" }
                if !headline_1.is_empty() { {icon_wrap(&headline_1)} }
                if !headline_2.is_empty() { {icon_wrap(&headline_2)} }
                if !body_1.is_empty() { {labeled(&body_1)} }
                if !body_2.is_empty() { {labeled(&body_2)} }
                if !footer.is_empty() { {icon_wrap(&footer)} }
                if !talents.is_empty() {
                    div { class: "filter-heading", "Talents" }
                    {icon_wrap(&talents)}
                }
            }
            {combos}
            div { class: "filter-clear",
                button { class: "btn btn-danger", onclick: move |_| { let open = (state.filter_open)(); state.filter.set(CatFilterState { is_open: open, ..CatFilterState::default() }); }, "Clear Filter" }
            }
        }
    }
}
