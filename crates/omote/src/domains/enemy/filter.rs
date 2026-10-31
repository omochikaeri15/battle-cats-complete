use std::collections::HashSet;
use std::rc::Rc;

use dioxus::prelude::*;

use codex::combat::{AttrUnit, Identity, REGISTRY};
use kokoro::domains::enemy::filter::evaluation::get_identity_name;
use kokoro::domains::enemy::filter::{ATTACK_TYPE_IDENTITIES, EnemyFilterState, MatchMode};
use kokoro::domains::enemy::scanner::EnemyEntry;
use kokoro::systems::combat::present_identities;
use kokoro::systems::combat::registry::{DisplayGroup, get_display_def};

use crate::domains::cat::icons::{self, ICON_SIZE};
use crate::systems::context::Shared;
use crate::systems::images;
use crate::widget::{Popup, RangeRow, Tip};

use super::list::Entries;
use super::state;

const STAT_KEYS: [&str; 8] = ["Attack", "Dps", "Range", "Atk Cycle (f)", "Hitpoints", "Knockbacks", "Speed", "Cash Drop"];

#[derive(Clone)]
pub(crate) struct Found(pub(crate) Rc<HashSet<Identity>>);

impl PartialEq for Found {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Found {
    pub(crate) fn of(enemies: &[EnemyEntry]) -> Self {
        Self(Rc::new(present_identities(enemies.iter().map(|enemy| &enemy.stats))))
    }
}

#[derive(Clone)]
pub(crate) struct Edited(pub(crate) EnemyFilterState);

impl PartialEq for Edited {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

fn ability_schema(identity: Identity) -> &'static [(&'static str, AttrUnit)] {
    REGISTRY.iter().find(|def| def.identity == identity).map_or(&[], |def| def.schema)
}

fn edited(base: &EnemyFilterState, on_change: EventHandler<EnemyFilterState>, edit: impl FnOnce(&mut EnemyFilterState)) {
    let mut next = base.clone();

    edit(&mut next);
    on_change.call(next);
}

fn toggle(filter: &mut EnemyFilterState, identity: Identity) {
    if !filter.active_identities.remove(&identity) {
        filter.active_identities.insert(identity);
    }
}

fn group_identities(available: &HashSet<Identity>, target: DisplayGroup) -> Vec<Identity> {
    let mut found = Vec::new();

    for def in REGISTRY.iter() {
        if !available.contains(&def.identity) {
            continue;
        }

        let display = get_display_def(def.identity);

        if display.group != target || display.group == DisplayGroup::Trait || ATTACK_TYPE_IDENTITIES.contains(&def.identity) || found.contains(&def.identity) {
            continue;
        }

        found.push(def.identity);
    }

    found
}

#[component]
pub(crate) fn FilterPopup(enemies: Entries, globals: Shared) -> Element {
    let state = state();
    let available = use_memo(use_reactive!(|enemies| Found::of(&enemies.0)));
    let filter = state.filter.read().clone();

    rsx! {
        Popup { title: "Advanced Enemy Filter", open: state.filter_open,
            Content {
                available: available(),
                globals,
                filter: Edited(filter),
                magnification_hint: "100",
                on_change: move |next: EnemyFilterState| {
                    let mut state = state;

                    state.filter.set(next);
                },
            }
        }
    }
}

#[component]
pub(crate) fn Content(available: Found, globals: Shared, filter: Edited, magnification_hint: &'static str, on_change: EventHandler<EnemyFilterState>) -> Element {
    let pictures = images::pictures();
    let filter = Rc::new(filter.0);
    let available = available.0;
    let types: Vec<Identity> = REGISTRY
        .iter()
        .map(|def| def.identity)
        .filter(|identity| available.contains(identity))
        .filter(|identity| get_display_def(*identity).group == DisplayGroup::Trait)
        .collect();
    let headline_1 = group_identities(&available, DisplayGroup::Headline1);
    let headline_2 = group_identities(&available, DisplayGroup::Headline2);
    let body_1 = group_identities(&available, DisplayGroup::Body1);
    let body_2 = group_identities(&available, DisplayGroup::Body2);
    let footer = group_identities(&available, DisplayGroup::Footer);
    let icon_wrap = |items: &[Identity]| {
        rsx! {
            div { class: "icon-row filter-icons",
                for identity in items.iter().copied() {
                    {
                        let active = filter.active_identities.contains(&identity);
                        let name = get_identity_name(identity, &globals.0.names);
                        let display = get_display_def(identity);
                        let base = Rc::clone(&filter);

                        rsx! {
                            Tip { text: name,
                                button {
                                    class: if active { "icon-toggle active" } else { "icon-toggle" },
                                    onclick: move |_| edited(&base, on_change, |next| toggle(next, identity)),
                                    {icons::ability_icon(&pictures, display.icon, ICON_SIZE, "?")}
                                }
                            }
                        }
                    }
                }
            }
        }
    };
    let labeled = |items: &[Identity]| {
        rsx! {
            div { class: "filter-labeled",
                for identity in items.iter().copied() {
                    {
                        let active = filter.active_identities.contains(&identity);
                        let name = get_identity_name(identity, &globals.0.names);
                        let display = get_display_def(identity);
                        let schema = ability_schema(identity);
                        let expanded = active && !schema.is_empty();
                        let base = Rc::clone(&filter);
                        let base_label = Rc::clone(&filter);

                        rsx! {
                            div { class: if expanded { "card filter-ability expanded" } else { "filter-ability" },
                                div { class: "filter-ability-header",
                                    button { class: if active { "icon-toggle active" } else { "icon-toggle" }, onclick: move |_| edited(&base, on_change, |next| toggle(next, identity)),
                                        {icons::ability_icon(&pictures, display.icon, ICON_SIZE, "?")}
                                    }
                                    button { class: "btn btn-text", onclick: move |_| edited(&base_label, on_change, |next| toggle(next, identity)), "{name}" }
                                }
                                if expanded {
                                    div { class: "filter-ranges",
                                        for (attr, _) in schema.iter().copied() {
                                            {
                                                let range = filter.adv_ranges.get(&identity).and_then(|ranges| ranges.get(attr));
                                                let min = range.map_or(String::new(), |range| range.min.clone());
                                                let max = range.map_or(String::new(), |range| range.max.clone());
                                                let base_min = Rc::clone(&filter);
                                                let base_max = Rc::clone(&filter);

                                                rsx! {
                                                    RangeRow {
                                                        label: attr,
                                                        min,
                                                        max,
                                                        on_min: move |value: String| edited(&base_min, on_change, |next| { next.adv_ranges.entry(identity).or_default().entry(attr).or_default().min = value; }),
                                                        on_max: move |value: String| edited(&base_max, on_change, |next| { next.adv_ranges.entry(identity).or_default().entry(attr).or_default().max = value; }),
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
    let base_mode = Rc::clone(&filter);
    let base_mag = Rc::clone(&filter);
    let base_clear = Rc::clone(&filter);

    rsx! {
        div { class: "filter-content",
            div { class: "filter-section",
                div { class: "filter-heading", "Attributes" }
                div { class: "mode-row",
                    label { "Mode:"
                        select { class: "combo", value: if filter.match_mode == MatchMode::And { "And" } else { "Or" },
                            onchange: move |event: Event<FormData>| edited(&base_mode, on_change, |next| next.match_mode = if event.value() == "And" { MatchMode::And } else { MatchMode::Or }),
                            option { value: "And", selected: filter.match_mode == MatchMode::And, "And" }
                            option { value: "Or", selected: filter.match_mode == MatchMode::Or, "Or" }
                        }
                    }
                }
            }
            div { class: "filter-section",
                div { class: "filter-heading", "Stats" }
                label { class: "level-filter", "Target Magnification:"
                    input { class: "input", r#type: "text", inputmode: "numeric", placeholder: "{magnification_hint}", value: "{filter.mag_input}",
                        oninput: move |event: Event<FormData>| edited(&base_mag, on_change, |next| next.mag_input = event.value()) }
                    span { "%" }
                }
                div { class: "stat-ranges",
                    for stat in STAT_KEYS {
                        {
                            let range = filter.stat_ranges.get(stat);
                            let min = range.map_or(String::new(), |range| range.min.clone());
                            let max = range.map_or(String::new(), |range| range.max.clone());
                            let base_min = Rc::clone(&filter);
                            let base_max = Rc::clone(&filter);

                            rsx! {
                                RangeRow {
                                    label: stat,
                                    min,
                                    max,
                                    on_min: move |value: String| edited(&base_min, on_change, |next| { next.stat_ranges.entry(stat).or_default().min = value; }),
                                    on_max: move |value: String| edited(&base_max, on_change, |next| { next.stat_ranges.entry(stat).or_default().max = value; }),
                                }
                            }
                        }
                    }
                }
            }
            div { class: "filter-section",
                div { class: "filter-heading", "Trait Type" }
                {icon_wrap(&types)}
            }
            div { class: "filter-section",
                div { class: "filter-heading", "Attack Type" }
                {icon_wrap(ATTACK_TYPE_IDENTITIES)}
            }
            div { class: "filter-section",
                div { class: "filter-heading", "Abilities" }
                if !headline_1.is_empty() { {icon_wrap(&headline_1)} }
                if !headline_2.is_empty() { {icon_wrap(&headline_2)} }
                if !body_1.is_empty() { {labeled(&body_1)} }
                if !body_2.is_empty() { {labeled(&body_2)} }
                if !footer.is_empty() { {icon_wrap(&footer)} }
            }
            div { class: "filter-clear",
                button {
                    class: "btn btn-danger",
                    onclick: move |_| edited(&base_clear, on_change, |next| *next = EnemyFilterState { is_open: next.is_open, treasure: next.treasure, ..EnemyFilterState::default() }),
                    "Clear Filter"
                }
            }
        }
    }
}
