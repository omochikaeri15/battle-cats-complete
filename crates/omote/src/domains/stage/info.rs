use std::collections::HashMap;

use dioxus::prelude::*;

use codex::chapter::Category;
use codex::chapter::map::LockSkipDataEntry;
use codex::chapter::stage::ScatCpuSetting;
use kokoro::domains::stage::{Map, Stage, StageDataState, cost, files as stage_files};

use crate::systems::context::Shared;
use crate::systems::images;
use crate::systems::picture::Pic;
use crate::widget::{StatHeader, StatValue};

use super::state;

const MAP_IMG_HEIGHT: f32 = 50.0;
const STAGE_IMG_HEIGHT: f32 = 35.0;
const CROWN_GLYPH: &str = "\u{1F732}";

fn format_diff(difficulty: u16) -> String {
    if difficulty == 0 { "-".to_owned() } else { format!("\u{2605}{difficulty}") }
}

fn format_base(anim_id: u32, standard_id: i32) -> (String, String) {
    if anim_id != 0 {
        return ("Base Enemy".to_owned(), format!("{:03}-E", anim_id.saturating_sub(2)));
    }

    ("Base Img".to_owned(), standard_id.to_string())
}

fn format_bool(status: bool, yes: &str, no: &str) -> String {
    if status { yes.to_owned() } else { no.to_owned() }
}

fn format_respawn(minimum: u32, maximum: u32) -> String {
    if minimum == maximum { format!("{minimum}f") } else { format!("{minimum}f ~ {maximum}f") }
}

fn format_boss_bgm(boss_track: i16, init_track: u32, change_percent: u32) -> String {
    if boss_track < 0 || boss_track as u32 == init_track || change_percent == 100 {
        return "-".to_owned();
    }

    boss_track.to_string()
}

fn format_time(time_limit: u32) -> String {
    if time_limit == 0 { "-".to_owned() } else { format!("{time_limit}m") }
}

fn skip_status(category: &Category, map_id: u32, lock_registry: &HashMap<u32, LockSkipDataEntry>, cpu_setting: &ScatCpuSetting) -> String {
    if let Some(global_id) = category.global_map_id(map_id)
        && let Some(entry) = lock_registry.get(&global_id)
        && entry.excluded_map_id == global_id
    {
        return "N/A".to_owned();
    }

    if cpu_setting.super_cpu_consume_amount > 0 {
        return format!("{} CPUs", cpu_setting.super_cpu_consume_amount);
    }

    "-".to_owned()
}

fn banner(globals: &Shared, name: &str, height: f32, fallback: &str) -> Element {
    let pictures = images::pictures();
    let vfs = &globals.0.vault().vfs;
    let sized = vfs.find(name).and_then(|path| pictures.trimmed(&path));

    match sized {
        Some(sized) => {
            let width = sized.width as f32 * (height / sized.height.max(1) as f32);

            rsx! { Pic { url: sized.url, class: "stage-banner", style: format!("width: {width}px; height: {height}px;"), alt: fallback.to_owned() } }
        }
        None => rsx! { span { class: "banner-fallback bold", "{fallback}" } },
    }
}

fn crowns(stage: &Stage, selected: u8) -> Element {
    let mut state = state();

    if stage.max_crowns <= 1 {
        return rsx! {};
    }

    rsx! {
        div { class: "crown-row",
            for crown in 0..stage.max_crowns {
                {
                    let enabled = stage.target_crowns == -1 || stage.target_crowns as u8 == crown;
                    let class = super::super::cat::header_class(selected == crown, enabled, false);

                    rsx! {
                        button { class: "{class} crown-btn", disabled: !enabled, onclick: move |_| state.crown.set(crown),
                            span { "{crown + 1}" }
                            span { class: "symbols", "{CROWN_GLYPH}" }
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn view(stage: &Stage, map: &Map, data: &StageDataState, globals: &Shared, crown: u8) -> Element {
    let vfs = &globals.0.vault().vfs;
    let prefix = stage.category.image_prefix();
    let map_banner = stage_files::map_banner_file(stage.map_id, &prefix);
    let stage_banner = stage_files::stage_banner_file(&stage.category, stage.map_id, stage.stage_id, &prefix);
    let magnification = stage.crown_magnification(crown.saturating_add(1)).unwrap_or(100);
    let final_hp = if stage.anim_base_id != 0 { (stage.base_hp * magnification) / 100 } else { stage.base_hp };
    let dojo = stage.category == Category::DojoRankingEvents || stage.category == Category::DojoHallOfInitiates;
    let hp_header = if dojo { "Time Limit" } else { "Base HP" };
    let hp_value = if dojo { format_time(stage.time_limit) } else { final_hp.to_string() };
    let resolved_cost = cost::resolve_cost(stage, &globals.0.vault().vds.items, vfs);
    let (base_header, base_value) = format_base(stage.anim_base_id, stage.base_id);
    let skip = skip_status(&stage.category, stage.map_id, &data.lock_skip_registry, &data.scat_cpu_setting);

    rsx! {
        div { class: "stage-info",
            div { class: "banner-row",
                {banner(globals, &map_banner, MAP_IMG_HEIGHT, &map.name)}
                {banner(globals, &stage_banner, STAGE_IMG_HEIGHT, &stage.name)}
            }
            div { class: "rule" }
            {crowns(stage, crown)}
            div { class: "section",
                div { class: "section-title", "Information" }
                div { class: "rule" }
                div { class: "stat-grid",
                    div { class: "stat-row",
                        StatHeader { label: hp_header }
                        StatHeader { label: "Width" }
                        StatHeader { label: resolved_cost.header.clone() }
                        StatHeader { label: "XP" }
                        StatHeader { label: "Boss Guard" }
                        StatHeader { label: "Max Enemy" }
                        StatHeader { label: "Respawn" }
                    }
                    div { class: "stat-row",
                        StatValue { header: hp_header, value: hp_value }
                        StatValue { header: "Width", value: stage.width.to_string() }
                        StatValue { header: resolved_cost.header.clone(), value: resolved_cost.value.clone() }
                        StatValue { header: "XP", value: stage.xp.to_string() }
                        StatValue { header: "Boss Guard", value: format_bool(stage.is_base_indestructible, "Active", "-") }
                        StatValue { header: "Max Enemy", value: stage.max_enemies.to_string() }
                        StatValue { header: "Respawn", value: format_respawn(stage.min_spawn, stage.max_spawn) }
                    }
                    div { class: "stat-row",
                        StatHeader { label: base_header.clone() }
                        StatHeader { label: "Background" }
                        StatHeader { label: "Music" }
                        StatHeader { label: "Boss Music" }
                        StatHeader { label: "Difficulty" }
                        StatHeader { label: "CPU Skip" }
                        StatHeader { label: "Continues" }
                    }
                    div { class: "stat-row",
                        StatValue { header: base_header.clone(), value: base_value }
                        StatValue { header: "Background", value: stage.background_id.to_string() }
                        StatValue { header: "Music", value: stage.init_track.to_string() }
                        StatValue { header: "Boss Music", value: format_boss_bgm(stage.boss_track, stage.init_track, stage.bgm_change_percent) }
                        StatValue { header: "Difficulty", value: format_diff(stage.difficulty) }
                        StatValue { header: "CPU Skip", value: skip }
                        StatValue { header: "Continues", value: format_bool(stage.is_no_continues, "No", "Yes") }
                    }
                }
            }
        }
    }
}
