use std::collections::HashMap;

use dioxus::prelude::*;

use codex::cat::unit::{Talent, TalentGroup};
use codex::combat::{Faction, get_talent};
use codex::files::img022;
use kokoro::domains::cat::game::stats::get_final_stats;
use kokoro::domains::cat::game::talents as talent_logic;
use kokoro::systems::combat::registry::{AbilityIcon, get_display_def};

use crate::systems::context::{self, Shared};
use crate::systems::images::{self, Pictures};
use crate::systems::picture::Pic;
use crate::widget::Superscript;

use super::icons::{self, IMG015, IMG022};
use super::{Picked, SelectedCat, state};

const HEADER_SCALE: f32 = 0.75;
const HEADER_ICON_SIZE: f32 = 40.0 * HEADER_SCALE;
const HEADER_NAME_HEIGHT: f32 = 35.0 * HEADER_SCALE;
const NP_ICON_SIZE: f32 = 20.0;
const HEADER_NP_ICON_SIZE: f32 = 24.0;

pub(crate) struct ToggleAvailability {
    pub(crate) has_normal_enabled: bool,
    pub(crate) has_ultra_enabled: bool,
    pub(crate) has_ultra_talents: bool,
}

pub(crate) fn toggle_availability(talent_data: &Talent, levels: Option<&HashMap<u8, u8>>) -> ToggleAvailability {
    let mut availability = ToggleAvailability { has_normal_enabled: false, has_ultra_enabled: false, has_ultra_talents: false };

    for (index, group) in talent_data.groups.iter().enumerate() {
        let level = levels.and_then(|levels| levels.get(&(index as u8))).copied().unwrap_or(0);

        if group.limit == 1 {
            availability.has_ultra_talents = true;
            availability.has_ultra_enabled |= level > 0;
        } else {
            availability.has_normal_enabled |= level > 0;
        }
    }

    availability
}

fn np_icon(pictures: &Pictures, size: f32) -> Element {
    match icons::sheet_url(pictures, IMG022, img022::ICON_NP_COST) {
        Some(url) => rsx! { Pic { url, class: "np-icon", style: format!("height: {size}px;"), alt: "NP" } },
        None => rsx! { span { class: "bold white", "NP Cost" } },
    }
}

#[component]
pub(crate) fn Header(cat: SelectedCat, globals: Shared) -> Element {
    let settings = context::settings();
    let mut state = state();
    let pictures = images::pictures();
    let Some(talent_data) = cat.0.talent_data.as_ref() else {
        return rsx! {};
    };
    let levels = state.levels_for(cat.0.id);
    let costs = globals.0.vault().vds.cats.talent_costs(&globals.0.vault().vfs);
    let total = talent_logic::get_total_np_cost(talent_data, &levels, &costs);
    let availability = toggle_availability(talent_data, Some(&levels));
    let normal_label = if availability.has_normal_enabled { "No Talents" } else { "All Talents" };
    let ultra_label = if availability.has_ultra_enabled { "No Ultra" } else { "All Ultra" };
    let for_normal = cat.clone();
    let for_ultra = cat.clone();

    rsx! {
        div { class: "talent-header",
            div { class: "np-row",
                {np_icon(&pictures, HEADER_NP_ICON_SIZE)}
                span { class: "np-total bold", "{total}" }
            }
            button { class: "btn btn-primary talent-btn", onclick: move |_| state.toggle_talents(&for_normal.0, false, &settings.read()), "{normal_label}" }
            button {
                class: "btn btn-primary talent-btn",
                disabled: !availability.has_ultra_talents,
                onclick: move |_| state.toggle_talents(&for_ultra.0, true, &settings.read()),
                "{ultra_label}"
            }
        }
    }
}

#[component]
pub(crate) fn Talents(selected: Picked, globals: Shared) -> Element {
    let state = state();
    let cat = &selected.0.cat;
    let Some(talent_data) = cat.talent_data.as_ref() else {
        return rsx! { div { class: "note", "No Talents Available" } };
    };
    let form = (state.form)();
    let levels = state.levels_for(cat.id);
    let base = selected.0.stats.as_ref().and_then(|stats| stats.get(form));
    let traits = base.map(|base| get_final_stats(base, cat.curve.as_ref(), (state.level)(), Some(talent_data), Some(&levels), &globals.0.treasure));

    rsx! {
        div { class: "talent-groups",
            for index in 0..talent_data.groups.len() {
                Group {
                    key: "{index}",
                    selected: selected.clone(),
                    globals: globals.clone(),
                    index: index as u8,
                    traits: traits.clone().map(Traits),
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct Traits(codex::combat::Entity);

#[component]
fn Group(selected: Picked, globals: Shared, index: u8, traits: Option<Traits>) -> Element {
    let mut state = state();
    let pictures = images::pictures();
    let cat = &selected.0.cat;
    let Some(talent_data) = cat.talent_data.as_ref() else {
        return rsx! {};
    };
    let Some(group) = talent_data.groups.get(usize::from(index)) else {
        return rsx! {};
    };
    let cat_id = cat.id;
    let expanded = state.expanded_groups.read().get(&(cat_id, index)).copied().unwrap_or(false);
    let class = if group.limit == 1 { "talent-group ultra" } else { "talent-group normal" };
    let arrow = if expanded { "\u{25b2}" } else { "\u{25bc}" };
    let icon = talent_icon(&pictures, group);
    let name = skill_name(&pictures, &globals, group);
    let fallback_name = globals.0.names.talent(group.ability_id).map(str::to_owned).unwrap_or_else(|| {
        get_talent(group.ability_id)
            .map_or_else(|| format!("Unknown Skill (ID: {})", group.ability_id), |def| globals.0.names.ability(def.identity, Faction::Cat).to_owned())
    });

    rsx! {
        div { class: "{class}",
            button {
                class: "talent-group-header",
                onclick: move |_| {
                    let mut groups = state.expanded_groups.write();
                    let next = !groups.get(&(cat_id, index)).copied().unwrap_or(false);

                    groups.insert((cat_id, index), next);
                },
                {icon}
                match name {
                    Some(url) => rsx! { Pic { url, class: "skill-name", style: format!("height: {HEADER_NAME_HEIGHT}px;"), alt: fallback_name.clone() } },
                    None => rsx! { span { class: "skill-name-text bold white", "{fallback_name}" } },
                }
                span { class: "talent-arrow symbols", "{arrow}" }
            }
            if expanded {
                {body(&selected, &globals, index, group, traits.as_ref().map(|traits| &traits.0), &pictures)}
            }
        }
    }
}

fn body(
    selected: &Picked,
    globals: &Shared,
    index: u8,
    group: &TalentGroup,
    traits: Option<&codex::combat::Entity>,
    pictures: &Pictures,
) -> Element {
    let settings = context::settings();
    let mut state = state();
    let cat = &selected.0.cat;
    let Some(talent_data) = cat.talent_data.as_ref() else {
        return rsx! {};
    };
    let vault = globals.0.vault();
    let descriptions = vault.vds.cats.descriptions(&vault.vfs);
    let costs = vault.vds.cats.talent_costs(&vault.vfs);
    let description = descriptions.get(usize::from(group.text_id)).cloned().unwrap_or_else(|| "No skill description found".to_owned());
    let levels = state.levels_for(cat.id);
    let current = levels.get(&index).copied().unwrap_or(0);
    let np_cost = talent_logic::get_talent_np_cost(group.cost_id, current, &costs);
    let max_level = group.max_level.max(1);
    let input = state.talent_inputs.read().get(&index).cloned().unwrap_or_else(|| current.to_string());
    let form = (state.form)();
    let unit_level = (state.level)();
    let shown = selected.0.stats.as_ref().and_then(|stats| stats.get(form)).map(|stats| {
        let traits = traits.unwrap_or(stats);
        let scaling = talent_logic::Scaling { curve: cat.curve.as_ref(), level: unit_level, treasure: &globals.0.treasure };

        talent_logic::calculate_linked_display(talent_data, &levels, usize::from(index), stats, scaling, traits)
    });
    let for_slider = selected.clone();
    let for_input = selected.clone();

    rsx! {
        div { class: "talent-body",
            div { class: "dark-box", Superscript { text: description, size: 13.0, white: true } }
            div { class: "dark-box np-row",
                {np_icon(pictures, NP_ICON_SIZE)}
                span { class: "bold white np-cost", "{np_cost}" }
            }
            div { class: "dark-box",
                div { class: "level-slider-row",
                    span { class: "bold white", "Level:" }
                    input {
                        class: "slider",
                        r#type: "range",
                        min: "0",
                        max: "{max_level}",
                        value: "{current}",
                        oninput: move |event| {
                            if let Ok(level) = event.value().parse::<u8>() {
                                state.set_talent_level(&for_slider.0.cat, index, level, &settings.read());
                            }
                        },
                    }
                    input {
                        class: "input talent-level-input",
                        r#type: "text",
                        inputmode: "numeric",
                        placeholder: "0",
                        value: "{input}",
                        readonly: state.locked,
                        oninput: move |event| state.set_talent_input(&for_input.0.cat, index, event.value(), &settings.read()),
                    }
                }
                if let Some(text) = shown.flatten() {
                    Superscript { text, size: 15.0, bold: true, white: true }
                }
            }
        }
    }
}

fn talent_icon(pictures: &Pictures, group: &TalentGroup) -> Element {
    let Some(def) = get_talent(group.ability_id) else {
        return rsx! { span { class: "white", "?" } };
    };
    let display = get_display_def(def.identity);
    let icon = match display.icon {
        AbilityIcon::Custom(custom) => icons::custom_url(pictures, custom),
        AbilityIcon::Standard(icon_id) => icons::sheet_url(pictures, IMG015, icon_id),
        AbilityIcon::None => None,
    };

    icon.map_or_else(|| icons::fallback(display.fallback, HEADER_ICON_SIZE), |url| icons::image(url, HEADER_ICON_SIZE))
}

fn skill_name(pictures: &Pictures, globals: &Shared, group: &TalentGroup) -> Option<std::rc::Rc<str>> {
    let image_id = if group.name_id > 0 { group.name_id } else { i16::from(group.ability_id) };

    if image_id <= 0 {
        return None;
    }

    let vfs = &globals.0.vault().vfs;
    let path = vfs.find(&format!("Skill_name_{image_id:03}.png"))?;

    pictures.trimmed(&path).map(|sized| sized.url)
}
