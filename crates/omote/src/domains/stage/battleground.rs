use dioxus::prelude::*;

use codex::chapter::map::{BonusType, RuleType, ScoreBonusMapEntry, SpecialRulesMapEntry};
use codex::chapter::stage::{BossType, EnemyAmount};
use codex::common::{BreakHandling, strip_html_tags};
use kokoro::common::context::GlobalContext;
use kokoro::domains::stage::{Map, Stage, StageDataState, restrictions};
use kokoro::systems::treasure;
use tracing::warn;

use crate::app::Route;
use crate::domains::enemy;
use crate::systems::context::Shared;
use crate::systems::images;
use crate::systems::picture::Pic;
use crate::widget::Tip;


fn format_amount(amount: &EnemyAmount) -> String {
    match amount {
        EnemyAmount::Infinite => "\u{221e}".to_owned(),
        EnemyAmount::Limit(limit) => limit.to_string(),
    }
}

fn format_spawn(time_flag: u32, start_frame: u32, base_hp_perc: u32) -> String {
    if base_hp_perc != 100 && time_flag == 0 { "-".to_owned() } else { format!("{start_frame}f") }
}

fn format_respawn(amount: &EnemyAmount, min: u32, max: u32) -> String {
    if amount == &EnemyAmount::Limit(1) {
        return "-".to_owned();
    }

    if min == max { format!("{min}f") } else { format!("{min}f ~ {max}f") }
}

fn format_layer(min: i32, max: i32) -> String {
    if min == max { min.to_string() } else { format!("{min} ~ {max}") }
}

fn format_boss(boss: &BossType) -> String {
    match boss {
        BossType::None => "-".to_owned(),
        BossType::Boss => "Yes".to_owned(),
        BossType::ScreenShake => "Yes (Shake)".to_owned(),
        BossType::Unknown(_) => "Unknown".to_owned(),
    }
}

fn format_count(count: u32) -> String {
    if count == 0 { "-".to_owned() } else { count.to_string() }
}

fn format_base_hp(base_hp_perc: u32, dojo: bool) -> String {
    if dojo {
        return base_hp_perc.to_string();
    }

    if base_hp_perc == 100 { "-".to_owned() } else { format!("{base_hp_perc}%") }
}

fn rule_parameters(rule: &RuleType) -> &[u32] {
    match rule {
        RuleType::TrustFund(params)
        | RuleType::CooldownEquality(params)
        | RuleType::LineupLimit(params)
        | RuleType::RarityLimit(params)
        | RuleType::CheapLabor(params)
        | RuleType::CatCost(params)
        | RuleType::CatProduction(params)
        | RuleType::TotalDeployLimit(params)
        | RuleType::MoreThanOne(params)
        | RuleType::MegaCatCannon(params)
        | RuleType::UniformMotion(params)
        | RuleType::CompoundingCost(params)
        | RuleType::Unknown(_, params) => params,
    }
}

fn rule_name(rule: &RuleType) -> String {
    match rule {
        RuleType::TrustFund(params) => format!("Trust Fund {params:?}"),
        RuleType::CooldownEquality(params) => format!("Cooldown Equality {params:?}"),
        RuleType::LineupLimit(params) => format!("Lineup Limit {params:?}"),
        RuleType::RarityLimit(params) => format!("Rarity Limit {params:?}"),
        RuleType::CheapLabor(params) => format!("Cheap Labor {params:?}"),
        RuleType::CatCost(params) => format!("Cat Cost {params:?}"),
        RuleType::CatProduction(params) => format!("Cat Production {params:?}"),
        RuleType::TotalDeployLimit(params) => format!("Total Deploy Limit {params:?}"),
        RuleType::MoreThanOne(params) => format!("More Than One {params:?}"),
        RuleType::MegaCatCannon(params) => format!("Mega Cat Cannon {params:?}"),
        RuleType::UniformMotion(params) => format!("Uniform Motion {params:?}"),
        RuleType::CompoundingCost(params) => format!("Compounding Cost {params:?}"),
        RuleType::Unknown(id, params) => format!("Unknown Rule {id} {params:?}"),
    }
}

fn special_rule(rule: &SpecialRulesMapEntry, ctx: &GlobalContext<'_>) -> String {
    let key = if !rule.explanation_label.is_empty() {
        rule.explanation_label.clone()
    } else if let Some(prefix) = rule.name_label.trim().strip_suffix("Name") {
        format!("{prefix}Explanation")
    } else {
        String::new()
    };
    let raw = if key.is_empty() { "" } else { ctx.localizable.lookup(&key).unwrap_or("") };
    let mut description = strip_html_tags(raw, BreakHandling::Space);

    if description.is_empty() {
        if !key.is_empty() {
            warn!(key = %key, name_label = %rule.name_label, "missing localization for special rule explanation, falling back to raw enum parsing");
        }

        return rule.rules.iter().map(rule_name).collect::<Vec<_>>().join("\n");
    }

    for target in &rule.rules {
        for param in rule_parameters(target) {
            description = description.replacen("%d", &param.to_string(), 1);
        }
    }

    description
}

fn bonus_parameters(bonus: &BonusType) -> &[u32] {
    match bonus {
        BonusType::Weaken(params)
        | BonusType::Freeze(params)
        | BonusType::Slow(params)
        | BonusType::Knockback(params)
        | BonusType::StrongAttack(params)
        | BonusType::MassiveDamage(params)
        | BonusType::StrongDefense(params)
        | BonusType::Resist(params)
        | BonusType::Unknown(_, params) => params,
    }
}

fn score_bonus(bonus: &ScoreBonusMapEntry, ctx: &GlobalContext<'_>) -> String {
    let key = if bonus.explanation_label.is_empty() { &bonus.name_label } else { &bonus.explanation_label };
    let raw = ctx.localizable.lookup(key).unwrap_or("");
    let mut description = strip_html_tags(raw, BreakHandling::Space);

    if description.is_empty() {
        return format!("\u{3010}{key}\u{3011} Localization data missing.");
    }

    for held in &bonus.bonuses {
        for param in bonus_parameters(held) {
            description = description.replacen("%d", &param.to_string(), 1);
        }
    }

    description
}

pub(crate) fn view(stage: &Stage, map: &Map, crown: u8, data: &StageDataState, globals: &Shared) -> Element {
    let pictures = images::pictures();
    let navigator = use_navigator();
    let enemy_state = enemy::state();
    let ctx = globals.0.context();
    let restrictions = restrictions::parse_restrictions(stage, crown as i8, ctx);
    let rule = map.special_rules.as_ref().map(|rule| special_rule(rule, &ctx));
    let bonus = map.score_bonuses.as_ref().map(|bonus| score_bonus(bonus, &ctx));
    let crown_mag = stage.crown_magnification(crown.saturating_add(1)).unwrap_or(100);
    let chapter = treasure::chapter(&stage.category, stage.map_id);
    let show_score = stage.enemies.iter().any(|enemy| enemy.score > 0);
    let dojo = stage.enemies.iter().any(|enemy| enemy.base_hp_perc > 100);
    let split = stage.enemies.iter().any(|enemy| (enemy.magnification * crown_mag) / 100 != (enemy.atk_magnification * crown_mag) / 100);
    let mag_detail = split.then_some("(HP% / ATK%)");
    let base_header = if dojo { "Dmg #" } else { "Base %" };

    rsx! {
        div { class: "section battleground",
            div { class: "section-title", "Battleground" }
            div { class: "rule" }
            if !restrictions.is_empty() {
                div { class: "section",
                    div { class: "subsection-title", "Restrictions" }
                    for line in restrictions.iter() {
                        div { "{line}" }
                    }
                }
            }
            if let Some(rule) = rule {
                div { class: "section",
                    div { class: "subsection-title", "Rules" }
                    pre { class: "details-text", "{rule}" }
                    if !map.invalid_combos.is_empty() {
                        div { "Disabled Combos: {map.invalid_combos.len()} total" }
                    }
                }
            }
            if let Some(bonus) = bonus {
                div { class: "section",
                    div { class: "subsection-title", "Score Bonus" }
                    div { "{bonus}" }
                }
            }
            if stage.enemies.is_empty() {
                div { "No enemies defined for this stage." }
            } else {
                div { class: "ground-scroll scroll",
                    table { class: "ground-table",
                        thead {
                            tr { class: "zebra-header",
                                th { "Enemy" }
                                th { "Count" }
                                th {
                                    "Magnification %"
                                    if let Some(detail) = mag_detail {
                                        br {}
                                        "{detail}"
                                    }
                                }
                                th { "{base_header}" }
                                th { "Spawn" }
                                th { "Respawn" }
                                th { "Layer" }
                                th { "Boss" }
                                if show_score {
                                    th { "Score" }
                                }
                                th { "Kills" }
                            }
                        }
                        tbody {
                            for (index, spawn) in stage.enemies.iter().enumerate() {
                                {
                                    let name = data.enemy_name_registry.get(spawn.enemy_id as usize).filter(|name| !name.is_empty()).cloned().unwrap_or_else(|| format!("{:03}-E", spawn.enemy_id));
                                    let hp_mag = (spawn.magnification * crown_mag) / 100;
                                    let atk_mag = (spawn.atk_magnification * crown_mag) / 100;
                                    let same = hp_mag == atk_mag;
                                    let entry = data.enemy_registry.get(&spawn.enemy_id);
                                    let icon = entry.and_then(|entry| entry.icon_path.as_ref()).and_then(|path| pictures.trimmed(path));
                                    let gap = entry.map_or(0, |entry| globals.0.treasure.weakening(&entry.stats, chapter));
                                    let strengthened = |magnification: u32| (i64::from(magnification) * i64::from(100 + gap) / 100).to_string();
                                    let formatted = if same { format!("{}%", strengthened(hp_mag)) } else { format!("{}% / {}%", strengthened(hp_mag), strengthened(atk_mag)) };
                                    let magnification_input = if same { hp_mag.to_string() } else { format!("{hp_mag}/{atk_mag}") };
                                    let enemy_id = spawn.enemy_id;
                                    let class = if index % 2 == 0 { "zebra-row-even" } else { "zebra-row-odd" };

                                    rsx! {
                                        tr { class: "{class}",
                                            td {
                                                Tip { text: name.clone(),
                                                    button {
                                                        class: "ground-enemy",
                                                        onclick: move |_| {
                                                            let mut enemy_state = enemy_state;

                                                            enemy_state.seeded_for.set(Some(enemy_id));
                                                            enemy_state.set_magnification(magnification_input.clone());
                                                            navigator.push(Route::enemy(enemy_id));
                                                        },
                                                        match icon.as_ref() {
                                                            Some(sized) => rsx! { Pic { url: sized.url.clone(), class: "drop-icon", alt: name.clone() } },
                                                            None => rsx! { span { class: "tiny", "{enemy_id:03}" } },
                                                        }
                                                    }
                                                }
                                            }
                                            td { "{format_amount(&spawn.amount)}" }
                                            td { "{formatted}" }
                                            td { "{format_base_hp(spawn.base_hp_perc, dojo)}" }
                                            td { "{format_spawn(spawn.time_flag, spawn.start_frame, spawn.base_hp_perc)}" }
                                            td { "{format_respawn(&spawn.amount, spawn.respawn_min, spawn.respawn_max)}" }
                                            td { "{format_layer(spawn.layer_min, spawn.layer_max)}" }
                                            td { "{format_boss(&spawn.boss_type)}" }
                                            if show_score {
                                                td { "{format_count(spawn.score)}" }
                                            }
                                            td { "{format_count(spawn.kill_count)}" }
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
