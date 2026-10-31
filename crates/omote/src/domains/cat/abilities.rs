use dioxus::prelude::*;

use codex::combat::Entity;
use codex::files::img015;
use kokoro::domains::cat::game::stats::get_final_stats;
use kokoro::domains::cat::waiter::unitid;
use kokoro::systems::combat::abilities::collect_ability_data;
use kokoro::systems::combat::registry::{
    Magnification, STAT_ATK_CYCLE, STAT_ATTACK, STAT_COOLDOWN, STAT_COST, STAT_DPS, STAT_HITPOINTS, STAT_KNOCKBACKS, STAT_RANGE, STAT_RARITY, STAT_SPEED,
    StatContext, format_stat,
};
use kokoro::systems::combat::{AbilityItem, CustomIcon, RenderContext};
use kokoro::systems::treasure::Bonus;

use crate::systems::context::{self, Shared};
use crate::systems::images::{self, Pictures};
use crate::widget::{StatFrames, StatHeader, StatValue, Superscript, Tip};

use super::icons::{self, ICON_SIZE, IMG015};
use super::{Picked, state};

pub(crate) const DESCRIPTION_TEXT_SIZE: f32 = 13.0;

#[component]
pub(crate) fn Abilities(selected: Picked, globals: Shared) -> Element {
    let settings = context::settings();
    let mut state = state();
    let pictures = images::pictures();
    let cat = &selected.0.cat;
    let form = (state.form)();
    let level = (state.level)();
    let Some(base) = selected.0.stats.as_ref().and_then(|stats| stats.get(form)) else {
        return rsx! { div { class: "note", "Stats data not found" } };
    };
    let allows_talents = form >= 2;
    let talent_data = if allows_talents { cat.talent_data.as_ref() } else { None };
    let levels = allows_talents.then(|| state.levels_for(cat.id));
    let final_stats = get_final_stats(base, cat.curve.as_ref(), level, talent_data, levels.as_ref(), &globals.0.treasure);
    let ctx = RenderContext {
        global: globals.0.context(),
        base_stats: base,
        final_stats: &final_stats,
        magnification: Magnification::default(),
        current_level: level,
        level_curve: cat.curve.as_ref(),
        talent_data,
        talent_levels: levels.as_ref(),
        is_conjure_unit: false,
    };
    let (traits, headline_1, headline_2, body_1, body_2, footer) = collect_ability_data(&ctx);
    let conjure_id = base.conjure_unit_id;
    let conjure_open = state.conjure_expanded.read().get(&cat.id).copied().unwrap_or(settings.read().cat_data.expand_spirit_details);
    let cat_id = cat.id;

    rsx! {
        div { class: "abilities",
            {stats_grid(cat, &final_stats, form, &globals.0.treasure)}
            div { class: "ability-groups",
                if !traits.is_empty() {
                    {icon_row(&pictures, &traits)}
                }
                if !headline_1.is_empty() {
                    {icon_row(&pictures, &headline_1)}
                }
                if !headline_2.is_empty() {
                    {icon_row(&pictures, &headline_2)}
                }
                for items in [&body_1, &body_2] {
                    if !items.is_empty() {
                        div { class: "ability-list",
                            for item in items.iter() {
                                if item.icon_id == Some(img015::ICON_CONJURE) && item.custom_icon == CustomIcon::None {
                                    div { class: "ability-row",
                                        {icons::item_icon(&pictures, item, ICON_SIZE)}
                                        Superscript { text: item.text.clone(), size: DESCRIPTION_TEXT_SIZE }
                                        button {
                                            class: if conjure_open { "btn btn-toggle active conjure-toggle" } else { "btn btn-toggle conjure-toggle" },
                                            onclick: move |_| {
                                                let next = !conjure_open;
                                                state.conjure_expanded.write().insert(cat_id, next);
                                            },
                                            "Details"
                                        }
                                    }
                                    if conjure_open {
                                        {spirit_card(&pictures, &globals, conjure_id, cat.curve.as_ref(), level)}
                                    }
                                } else {
                                    {ability_row(&pictures, item)}
                                }
                            }
                        }
                    }
                }
                if !footer.is_empty() {
                    {icon_row(&pictures, &footer)}
                }
            }
        }
    }
}

fn stats_grid(cat: &kokoro::domains::cat::scanner::CatEntry, final_stats: &Entity, form: usize, treasure: &Bonus) -> Element {
    let ctx = StatContext::cat(final_stats, cat.atk_anim_frames[form], Some(&cat.unitbuy), treasure);
    let cycle = (STAT_ATK_CYCLE.get_value)(&ctx);
    let cooldown = (STAT_COOLDOWN.get_value)(&ctx);

    rsx! {
        div { class: "stat-grid",
            div { class: "stat-row",
                StatHeader { label: STAT_ATTACK.display_name }
                StatHeader { label: STAT_DPS.display_name }
                StatHeader { label: STAT_RANGE.display_name }
                StatHeader { label: STAT_ATK_CYCLE.display_name }
                StatHeader { label: STAT_RARITY.display_name }
            }
            div { class: "stat-row",
                StatValue { header: STAT_ATTACK.display_name, value: format_stat(&STAT_ATTACK, &ctx) }
                StatValue { header: STAT_DPS.display_name, value: format_stat(&STAT_DPS, &ctx) }
                StatValue { header: STAT_RANGE.display_name, value: format_stat(&STAT_RANGE, &ctx) }
                StatFrames { header: STAT_ATK_CYCLE.display_name, frames: cycle }
                StatValue { header: STAT_RARITY.display_name, value: format_stat(&STAT_RARITY, &ctx) }
            }
            div { class: "stat-row",
                StatHeader { label: STAT_HITPOINTS.display_name }
                StatHeader { label: STAT_KNOCKBACKS.display_name }
                StatHeader { label: STAT_SPEED.display_name }
                StatHeader { label: STAT_COOLDOWN.display_name }
                StatHeader { label: STAT_COST.display_name }
            }
            div { class: "stat-row",
                StatValue { header: STAT_HITPOINTS.display_name, value: format_stat(&STAT_HITPOINTS, &ctx) }
                StatValue { header: STAT_KNOCKBACKS.display_name, value: format_stat(&STAT_KNOCKBACKS, &ctx) }
                StatValue { header: STAT_SPEED.display_name, value: format_stat(&STAT_SPEED, &ctx) }
                StatFrames { header: STAT_COOLDOWN.display_name, frames: cooldown }
                StatValue { header: STAT_COST.display_name, value: format_stat(&STAT_COST, &ctx) }
            }
        }
    }
}

pub(crate) fn icon_row(pictures: &Pictures, items: &[AbilityItem]) -> Element {
    rsx! {
        div { class: "icon-row",
            for item in items {
                Tip { text: item.text.clone(), {icons::item_icon(pictures, item, ICON_SIZE)} }
            }
        }
    }
}

pub(crate) fn ability_row(pictures: &Pictures, item: &AbilityItem) -> Element {
    rsx! {
        div { class: "ability-row",
            {icons::item_icon(pictures, item, ICON_SIZE)}
            Superscript { text: item.text.clone(), size: DESCRIPTION_TEXT_SIZE }
        }
    }
}

fn spirit_card(pictures: &Pictures, globals: &Shared, conjure_id: i32, curve: Option<&codex::cat::unit::LevelCurve>, level: i32) -> Element {
    let vfs = &globals.0.vault().vfs;
    let Some(stats) = unitid(vfs, conjure_id).and_then(|forms| forms.into_iter().next()) else {
        return rsx! { div { class: "note", "Spirit data not found" } };
    };
    let final_stats = get_final_stats(&stats, curve, level, None, None, &globals.0.treasure);
    let ctx = RenderContext {
        global: globals.0.context(),
        base_stats: &stats,
        final_stats: &final_stats,
        magnification: Magnification::default(),
        current_level: level,
        level_curve: curve,
        talent_data: None,
        talent_levels: None,
        is_conjure_unit: true,
    };
    let (traits, headline_1, headline_2, body_1, body_2, footer) = collect_ability_data(&ctx);
    let area = icons::sheet_url(pictures, IMG015, img015::ICON_AREA_ATTACK);
    let damage = format!("Damage {}\nRange {}", final_stats.attack_1_damage, final_stats.standing_range);

    rsx! {
        div { class: "card spirit-card",
            div { class: "ability-row",
                {area.map_or_else(|| icons::fallback("AoE", ICON_SIZE), |url| icons::image(url, ICON_SIZE))}
                Superscript { text: damage, size: DESCRIPTION_TEXT_SIZE }
            }
            if !traits.is_empty() {
                {icon_row(pictures, &traits)}
            }
            if !headline_1.is_empty() {
                {icon_row(pictures, &headline_1)}
            }
            if !headline_2.is_empty() {
                {icon_row(pictures, &headline_2)}
            }
            for items in [&body_1, &body_2] {
                if !items.is_empty() {
                    div { class: "ability-list",
                        for item in items.iter() {
                            {ability_row(pictures, item)}
                        }
                    }
                }
            }
            if !footer.is_empty() {
                {icon_row(pictures, &footer)}
            }
        }
    }
}
