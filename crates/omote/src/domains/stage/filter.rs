use dioxus::prelude::*;

use codex::chapter::stage::{CataminGrade, CostType};
use kokoro::domains::enemy::filter::EnemyFilterState;
use kokoro::domains::stage::cost::grade_label;
use kokoro::domains::stage::filter::StageFilterState;
use kokoro::domains::stage::filter::enemy::EnemyFilter;
use kokoro::domains::stage::filter::lineup::LineupFilter;
use kokoro::domains::stage::filter::material::MaterialFilter;
use kokoro::domains::stage::filter::range::StatRange;
use kokoro::domains::stage::filter::treasure::TreasureFilter;
use kokoro::systems::combat::NameBook;

use crate::domains::enemy::filter::{Content, Edited, Found};
use crate::systems::context::Shared;
use crate::widget::{Popup, RangeRow};

use super::{Loaded, StageState, state};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Flag {
    Continues,
    BossGuard,
    UseSuperCpu,
    RuleTrustFund,
    RuleCooldownEquality,
    RuleRarityLimit,
    RuleCheapLabor,
    RuleCatCost,
    RuleCatProduction,
    RuleTotalDeployLimit,
    RuleMoreThanOne,
    RuleMegaCatCannon,
    RuleUniformMotion,
    InvalidCombos,
    BonusWeaken,
    BonusFreeze,
    BonusSlow,
    BonusKnockback,
    BonusStrongAttack,
    BonusMassiveDamage,
    BonusStrongDefense,
    BonusResist,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Range {
    Width,
    BaseHp,
    MaxEnemies,
    TimeLimit,
    Energy,
    ItemCost,
    ItemId,
    CataminCost,
    Xp,
    MinSpawn,
    MaxSpawn,
    Difficulty,
    MaxCrowns,
    TargetCrowns,
    MinCost,
    MaxCost,
    DeployLimit,
    AllowedRows,
    BaseId,
    AnimBaseId,
    BackgroundId,
    InitTrack,
    BossTrack,
    BgmChangePercent,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EnemyRange {
    Amount,
    StartFrame,
    RespawnMin,
    RespawnMax,
    BaseHpPerc,
    LayerMin,
    LayerMax,
    Magnification,
    AtkMagnification,
    Score,
    TimeFlag,
    KillCount,
}

const GENERAL_RULES: [Flag; 3] = [Flag::Continues, Flag::BossGuard, Flag::UseSuperCpu];
const SPECIAL_RULES: [Flag; 11] = [
    Flag::RuleTrustFund,
    Flag::RuleCooldownEquality,
    Flag::RuleRarityLimit,
    Flag::RuleCheapLabor,
    Flag::RuleCatCost,
    Flag::RuleCatProduction,
    Flag::RuleTotalDeployLimit,
    Flag::RuleMoreThanOne,
    Flag::RuleMegaCatCannon,
    Flag::RuleUniformMotion,
    Flag::InvalidCombos,
];
const SCORE_BONUSES: [Flag; 8] = [
    Flag::BonusWeaken,
    Flag::BonusFreeze,
    Flag::BonusSlow,
    Flag::BonusKnockback,
    Flag::BonusStrongAttack,
    Flag::BonusMassiveDamage,
    Flag::BonusStrongDefense,
    Flag::BonusResist,
];
const STATS: [Range; 10] = [
    Range::BaseHp,
    Range::Width,
    Range::TimeLimit,
    Range::MaxEnemies,
    Range::Xp,
    Range::Difficulty,
    Range::MaxCrowns,
    Range::TargetCrowns,
    Range::MinSpawn,
    Range::MaxSpawn,
];
const RESTRICTIONS: [Range; 4] = [Range::DeployLimit, Range::AllowedRows, Range::MinCost, Range::MaxCost];
const IDS_AUDIO: [Range; 6] = [Range::BaseId, Range::AnimBaseId, Range::BackgroundId, Range::InitTrack, Range::BossTrack, Range::BgmChangePercent];
const ENEMY_RANGES: [EnemyRange; 12] = [
    EnemyRange::Amount,
    EnemyRange::StartFrame,
    EnemyRange::RespawnMin,
    EnemyRange::RespawnMax,
    EnemyRange::BaseHpPerc,
    EnemyRange::LayerMin,
    EnemyRange::LayerMax,
    EnemyRange::Magnification,
    EnemyRange::AtkMagnification,
    EnemyRange::Score,
    EnemyRange::TimeFlag,
    EnemyRange::KillCount,
];

fn flag_mut(state: &mut StageFilterState, flag: Flag) -> &mut Option<bool> {
    match flag {
        Flag::Continues => &mut state.continues,
        Flag::BossGuard => &mut state.boss_guard,
        Flag::UseSuperCpu => &mut state.use_super_cpu,
        Flag::RuleTrustFund => &mut state.rule_trust_fund,
        Flag::RuleCooldownEquality => &mut state.rule_cooldown_equality,
        Flag::RuleRarityLimit => &mut state.rule_rarity_limit,
        Flag::RuleCheapLabor => &mut state.rule_cheap_labor,
        Flag::RuleCatCost => &mut state.rule_cat_cost,
        Flag::RuleCatProduction => &mut state.rule_cat_production,
        Flag::RuleTotalDeployLimit => &mut state.rule_total_deploy_limit,
        Flag::RuleMoreThanOne => &mut state.rule_more_than_one,
        Flag::RuleMegaCatCannon => &mut state.rule_mega_cat_cannon,
        Flag::RuleUniformMotion => &mut state.rule_uniform_motion,
        Flag::InvalidCombos => &mut state.invalid_combos,
        Flag::BonusWeaken => &mut state.bonus_weaken,
        Flag::BonusFreeze => &mut state.bonus_freeze,
        Flag::BonusSlow => &mut state.bonus_slow,
        Flag::BonusKnockback => &mut state.bonus_knockback,
        Flag::BonusStrongAttack => &mut state.bonus_strong_attack,
        Flag::BonusMassiveDamage => &mut state.bonus_massive_damage,
        Flag::BonusStrongDefense => &mut state.bonus_strong_defense,
        Flag::BonusResist => &mut state.bonus_resist,
    }
}

fn flag_label(flag: Flag, names: &NameBook) -> &str {
    match flag {
        Flag::Continues => "Continues",
        Flag::BossGuard => "Boss Guard",
        Flag::UseSuperCpu => "Use Super CPU",
        Flag::RuleTrustFund => "Trust Fund",
        Flag::RuleCooldownEquality => "Cooldown Equality",
        Flag::RuleRarityLimit => "Rarity Limit",
        Flag::RuleCheapLabor => "Cheap Labor",
        Flag::RuleCatCost => "Cat Cost",
        Flag::RuleCatProduction => "Cat Production",
        Flag::RuleTotalDeployLimit => "Total Deploy Limit",
        Flag::RuleMoreThanOne => "More Than One",
        Flag::RuleMegaCatCannon => "Mega Cat Cannon",
        Flag::RuleUniformMotion => "Uniform Motion",
        Flag::InvalidCombos => "Invalid Combos",
        Flag::BonusWeaken => names.bonus(0).unwrap_or("Weaken"),
        Flag::BonusKnockback => names.bonus(1).unwrap_or("Knockback"),
        Flag::BonusStrongAttack => names.bonus(2).unwrap_or("Strong Attack"),
        Flag::BonusFreeze => names.bonus(3).unwrap_or("Freeze"),
        Flag::BonusMassiveDamage => names.bonus(4).unwrap_or("Massive Damage"),
        Flag::BonusSlow => names.bonus(5).unwrap_or("Slow"),
        Flag::BonusStrongDefense => names.bonus(6).unwrap_or("Strong Defense"),
        Flag::BonusResist => names.bonus(7).unwrap_or("Resist"),
    }
}

fn range_mut(state: &mut StageFilterState, range: Range) -> &mut StatRange {
    match range {
        Range::Width => &mut state.width,
        Range::BaseHp => &mut state.base_hp,
        Range::MaxEnemies => &mut state.max_enemies,
        Range::TimeLimit => &mut state.time_limit,
        Range::Energy => &mut state.cost.energy,
        Range::ItemCost => &mut state.cost.item_cost,
        Range::ItemId => &mut state.cost.item_id,
        Range::CataminCost => &mut state.cost.catamin_cost,
        Range::Xp => &mut state.xp,
        Range::MinSpawn => &mut state.min_spawn,
        Range::MaxSpawn => &mut state.max_spawn,
        Range::Difficulty => &mut state.difficulty,
        Range::MaxCrowns => &mut state.max_crowns,
        Range::TargetCrowns => &mut state.target_crowns,
        Range::MinCost => &mut state.min_cost,
        Range::MaxCost => &mut state.max_cost,
        Range::DeployLimit => &mut state.deploy_limit,
        Range::AllowedRows => &mut state.allowed_rows,
        Range::BaseId => &mut state.base_id,
        Range::AnimBaseId => &mut state.anim_base_id,
        Range::BackgroundId => &mut state.background_id,
        Range::InitTrack => &mut state.init_track,
        Range::BossTrack => &mut state.boss_track,
        Range::BgmChangePercent => &mut state.bgm_change_percent,
    }
}

fn range_label(range: Range) -> &'static str {
    match range {
        Range::Width => "Width",
        Range::BaseHp => "Base HP",
        Range::MaxEnemies => "Max Enemies",
        Range::TimeLimit => "Time Limit (f)",
        Range::Energy | Range::ItemCost | Range::CataminCost => "Cost",
        Range::ItemId => "ID",
        Range::Xp => "XP Reward",
        Range::MinSpawn => "Min Spawn",
        Range::MaxSpawn => "Max Spawn",
        Range::Difficulty => "Difficulty",
        Range::MaxCrowns => "Max Crowns",
        Range::TargetCrowns => "Target Crowns",
        Range::MinCost => "Min Cost",
        Range::MaxCost => "Max Cost",
        Range::DeployLimit => "Deploy Limit",
        Range::AllowedRows => "Allowed Rows",
        Range::BaseId => "Base ID",
        Range::AnimBaseId => "Anim Base ID",
        Range::BackgroundId => "Background ID",
        Range::InitTrack => "Init Track",
        Range::BossTrack => "Boss Track",
        Range::BgmChangePercent => "BGM Change (%)",
    }
}

fn enemy_range_mut(filter: &mut EnemyFilter, range: EnemyRange) -> &mut StatRange {
    match range {
        EnemyRange::Amount => &mut filter.amount,
        EnemyRange::StartFrame => &mut filter.start_frame,
        EnemyRange::RespawnMin => &mut filter.respawn_min,
        EnemyRange::RespawnMax => &mut filter.respawn_max,
        EnemyRange::BaseHpPerc => &mut filter.base_hp_perc,
        EnemyRange::LayerMin => &mut filter.layer_min,
        EnemyRange::LayerMax => &mut filter.layer_max,
        EnemyRange::Magnification => &mut filter.magnification,
        EnemyRange::AtkMagnification => &mut filter.atk_magnification,
        EnemyRange::Score => &mut filter.score,
        EnemyRange::TimeFlag => &mut filter.time_flag,
        EnemyRange::KillCount => &mut filter.kill_count,
    }
}

fn enemy_range_label(range: EnemyRange) -> &'static str {
    match range {
        EnemyRange::Amount => "Amount",
        EnemyRange::StartFrame => "Start Frame",
        EnemyRange::RespawnMin => "Respawn Min",
        EnemyRange::RespawnMax => "Respawn Max",
        EnemyRange::BaseHpPerc => "Base HP (%)",
        EnemyRange::LayerMin => "Layer Min",
        EnemyRange::LayerMax => "Layer Max",
        EnemyRange::Magnification => "Magnification",
        EnemyRange::AtkMagnification => "Atk Mag",
        EnemyRange::Score => "Score",
        EnemyRange::TimeFlag => "Time Flag",
        EnemyRange::KillCount => "Kill Count",
    }
}

fn cycle(value: Option<bool>) -> Option<bool> {
    match value {
        None => Some(true),
        Some(true) => Some(false),
        Some(false) => None,
    }
}

fn edit(mut state: StageState, change: impl FnOnce(&mut StageFilterState)) {
    change(&mut state.filter.write());
}

fn tristate(value: Option<bool>, onclick: impl FnMut(Event<MouseData>) + 'static) -> Element {
    let (label, class) = match value {
        Some(true) => ("Yes", "btn btn-primary tristate"),
        Some(false) => ("No", "btn btn-danger tristate"),
        None => ("Any", "btn btn-neutral tristate"),
    };

    rsx! {
        button { class: "{class}", onclick, "{label}" }
    }
}

fn flags(items: &[Flag], filter: &StageFilterState, names: &NameBook, state: StageState) -> Element {
    rsx! {
        div { class: "flag-grid",
            for flag in items.iter().copied() {
                {
                    let mut probe = filter.clone();
                    let value = *flag_mut(&mut probe, flag);

                    rsx! {
                        label { class: "flag-row",
                            span { class: "flag-label", "{flag_label(flag, names)}:" }
                            {tristate(value, move |_| edit(state, |next| { let slot = flag_mut(next, flag); *slot = cycle(*slot); }))}
                        }
                    }
                }
            }
        }
    }
}

fn ranges(items: &[Range], filter: &StageFilterState, state: StageState) -> Element {
    rsx! {
        div { class: "stat-ranges",
            for range in items.iter().copied() {
                {
                    let mut probe = filter.clone();
                    let value = range_mut(&mut probe, range).clone();

                    rsx! {
                        RangeRow {
                            label: range_label(range),
                            min: value.min,
                            max: value.max,
                            on_min: move |text| edit(state, |next| range_mut(next, range).min = text),
                            on_max: move |text| edit(state, |next| range_mut(next, range).max = text),
                        }
                    }
                }
            }
        }
    }
}

fn name_field(label: &str, value: &str, state: StageState, apply: fn(&mut StageFilterState, String)) -> Element {
    rsx! {
        label { class: "name-field",
            span { class: "name-label", "{label}" }
            input { class: "input name-input", r#type: "text", placeholder: "Any", value: "{value}", oninput: move |event| edit(state, |next| apply(next, event.value())) }
        }
    }
}

fn mode_select(is_exclude: bool, onchange: impl FnMut(Event<FormData>) + 'static) -> Element {
    rsx! {
        select { class: "combo", value: if is_exclude { "Exclude" } else { "Include" }, onchange,
            option { value: "Include", selected: !is_exclude, "Include" }
            option { value: "Exclude", selected: is_exclude, "Exclude" }
        }
    }
}

fn cost_fields(filter: &StageFilterState, state: StageState) -> Element {
    let cost = &filter.cost;
    let type_label = match cost.cost_type {
        Some(CostType::Item) => "Item",
        Some(CostType::Catamin) => "Catamin",
        Some(_) => "Energy",
        None => "Any",
    };
    let grade = cost.catamin_grade.map_or("Any", grade_label);

    rsx! {
        div { class: "section",
            label { class: "labeled",
                span { class: "cost-label", "Type:" }
                select { class: "combo", value: "{type_label}",
                    onchange: move |event| edit(state, |next| next.cost.cost_type = match event.value().as_str() {
                        "Energy" => Some(CostType::Energy),
                        "Item" => Some(CostType::Item),
                        "Catamin" => Some(CostType::Catamin),
                        _ => None,
                    }),
                    for option in ["Any", "Energy", "Item", "Catamin"] {
                        option { value: "{option}", selected: option == type_label, "{option}" }
                    }
                }
            }
            match cost.cost_type {
                None => rsx! {},
                Some(CostType::Item) => ranges(&[Range::ItemCost, Range::ItemId], filter, state),
                Some(CostType::Catamin) => rsx! {
                    div { class: "stat-ranges",
                        {ranges(&[Range::CataminCost], filter, state)}
                        label { class: "labeled",
                            span { class: "cost-label", "Grade:" }
                            select { class: "combo", value: "{grade}",
                                onchange: move |event| edit(state, |next| next.cost.catamin_grade = match event.value().as_str() {
                                    "A" => Some(CataminGrade::A),
                                    "B" => Some(CataminGrade::B),
                                    "C" => Some(CataminGrade::C),
                                    _ => None,
                                }),
                                for option in ["Any", "A", "B", "C"] {
                                    option { value: "{option}", selected: option == grade, "{option}" }
                                }
                            }
                        }
                    }
                },
                Some(_) => ranges(&[Range::Energy], filter, state),
            }
        }
    }
}

fn enemy_card(index: usize, enemy: &EnemyFilter, state: StageState) -> Element {
    let boss_label = match enemy.boss_type {
        None => "Any",
        Some(0) => "None",
        Some(1) => "Boss",
        Some(2) => "Screen Shake",
        Some(_) => "Unknown",
    };
    let attributes_active = enemy.attributes.is_active();

    rsx! {
        div { class: "card filter-card",
            div { class: "card-head",
                {mode_select(enemy.is_exclude, move |event| edit(state, |next| { if let Some(enemy) = next.enemies.get_mut(index) { enemy.is_exclude = event.value() == "Exclude"; } }))}
                button { class: "btn btn-danger remove-btn", onclick: move |_| edit(state, |next| { if index < next.enemies.len() { next.enemies.remove(index); } }), "X" }
            }
            button {
                class: if attributes_active { "btn btn-toggle active small-btn" } else { "btn btn-toggle small-btn" },
                onclick: move |_| {
                    let mut state = state;

                    state.attributes_for.set(Some(index));
                },
                "Attributes"
            }
            div { class: "labeled-row",
                span { "Boss Type:" }
                select { class: "combo", value: "{boss_label}",
                    onchange: move |event| edit(state, |next| {
                        if let Some(enemy) = next.enemies.get_mut(index) {
                            enemy.boss_type = match event.value().as_str() {
                                "None" => Some(0),
                                "Boss" => Some(1),
                                "Screen Shake" => Some(2),
                                _ => None,
                            };
                        }
                    }),
                    for option in ["Any", "None", "Boss", "Screen Shake"] {
                        option { value: "{option}", selected: option == boss_label, "{option}" }
                    }
                }
                span { "Is Base:" }
                {tristate(enemy.is_base, move |_| edit(state, |next| { if let Some(enemy) = next.enemies.get_mut(index) { enemy.is_base = cycle(enemy.is_base); } }))}
            }
            label { class: "name-field",
                span { class: "name-label wide", "Name or ID:" }
                input { class: "input name-input", r#type: "text", placeholder: "Any", value: "{enemy.name_or_id}",
                    oninput: move |event| edit(state, |next| { if let Some(enemy) = next.enemies.get_mut(index) { enemy.name_or_id = event.value(); } }) }
            }
            div { class: "stat-ranges",
                for range in ENEMY_RANGES {
                    {
                        let mut probe = enemy.clone();
                        let value = enemy_range_mut(&mut probe, range).clone();

                        rsx! {
                            RangeRow {
                                label: enemy_range_label(range),
                                min: value.min,
                                max: value.max,
                                on_min: move |text| edit(state, |next| { if let Some(enemy) = next.enemies.get_mut(index) { enemy_range_mut(enemy, range).min = text; } }),
                                on_max: move |text| edit(state, |next| { if let Some(enemy) = next.enemies.get_mut(index) { enemy_range_mut(enemy, range).max = text; } }),
                            }
                        }
                    }
                }
            }
        }
    }
}

fn lineup_card(index: usize, cat: &LineupFilter, state: StageState) -> Element {
    rsx! {
        div { class: "card filter-card",
            div { class: "card-head",
                {mode_select(cat.is_exclude, move |event| edit(state, |next| { if let Some(cat) = next.lineup_cats.get_mut(index) { cat.is_exclude = event.value() == "Exclude"; } }))}
                button { class: "btn btn-danger remove-btn", onclick: move |_| edit(state, |next| { if index < next.lineup_cats.len() { next.lineup_cats.remove(index); } }), "X" }
            }
            label { class: "name-field",
                span { class: "name-label wide", "Name or ID:" }
                input { class: "input name-input", r#type: "text", placeholder: "Any", value: "{cat.name_or_id}",
                    oninput: move |event| edit(state, |next| { if let Some(cat) = next.lineup_cats.get_mut(index) { cat.name_or_id = event.value(); } }) }
            }
            RangeRow {
                label: "Total Level",
                min: cat.level.min.clone(),
                max: cat.level.max.clone(),
                on_min: move |text| edit(state, |next| { if let Some(cat) = next.lineup_cats.get_mut(index) { cat.level.min = text; } }),
                on_max: move |text| edit(state, |next| { if let Some(cat) = next.lineup_cats.get_mut(index) { cat.level.max = text; } }),
            }
        }
    }
}

fn treasure_card(index: usize, treasure: &TreasureFilter, state: StageState) -> Element {
    rsx! {
        div { class: "card filter-card",
            div { class: "card-head",
                {mode_select(treasure.is_exclude, move |event| edit(state, |next| { if let Some(treasure) = next.treasures.get_mut(index) { treasure.is_exclude = event.value() == "Exclude"; } }))}
                button { class: "btn btn-danger remove-btn", onclick: move |_| edit(state, |next| { if index < next.treasures.len() { next.treasures.remove(index); } }), "X" }
            }
            label { class: "name-field",
                span { class: "name-label wide", "Name or ID:" }
                input { class: "input name-input", r#type: "text", placeholder: "Any", value: "{treasure.name_or_id}",
                    oninput: move |event| edit(state, |next| { if let Some(treasure) = next.treasures.get_mut(index) { treasure.name_or_id = event.value(); } }) }
            }
            RangeRow {
                label: "Amount",
                min: treasure.amount.min.clone(),
                max: treasure.amount.max.clone(),
                on_min: move |text| edit(state, |next| { if let Some(treasure) = next.treasures.get_mut(index) { treasure.amount.min = text; } }),
                on_max: move |text| edit(state, |next| { if let Some(treasure) = next.treasures.get_mut(index) { treasure.amount.max = text; } }),
            }
            RangeRow {
                label: "Chance (%)",
                min: treasure.chance.min.clone(),
                max: treasure.chance.max.clone(),
                on_min: move |text| edit(state, |next| { if let Some(treasure) = next.treasures.get_mut(index) { treasure.chance.min = text; } }),
                on_max: move |text| edit(state, |next| { if let Some(treasure) = next.treasures.get_mut(index) { treasure.chance.max = text; } }),
            }
        }
    }
}

fn material_card(index: usize, material: &MaterialFilter, state: StageState) -> Element {
    rsx! {
        div { class: "card filter-card",
            div { class: "card-head",
                {mode_select(material.is_exclude, move |event| edit(state, |next| { if let Some(material) = next.materials.get_mut(index) { material.is_exclude = event.value() == "Exclude"; } }))}
                button { class: "btn btn-danger remove-btn", onclick: move |_| edit(state, |next| { if index < next.materials.len() { next.materials.remove(index); } }), "X" }
            }
            label { class: "name-field",
                span { class: "name-label wide", "Name or ID:" }
                input { class: "input name-input", r#type: "text", placeholder: "Any", value: "{material.name_or_id}",
                    oninput: move |event| edit(state, |next| { if let Some(material) = next.materials.get_mut(index) { material.name_or_id = event.value(); } }) }
            }
            RangeRow {
                label: "Amount",
                min: material.amount.min.clone(),
                max: material.amount.max.clone(),
                on_min: move |text| edit(state, |next| { if let Some(material) = next.materials.get_mut(index) { material.amount.min = text; } }),
                on_max: move |text| edit(state, |next| { if let Some(material) = next.materials.get_mut(index) { material.amount.max = text; } }),
            }
        }
    }
}

#[component]
pub(crate) fn FilterPopup(data: Loaded, globals: Shared) -> Element {
    let mut state = state();
    let filter = state.filter.read().clone();
    let names = &globals.0.names;
    let available = use_memo(use_reactive!(|data| {
        let enemies: Vec<_> = data.0.state.enemy_registry.values().cloned().collect();

        Found::of(&enemies)
    }));
    let attributes_for = (state.attributes_for)();
    let attributes_open = use_signal(|| false);
    let mut attributes_open = attributes_open;

    if attributes_for.is_some() != attributes_open() {
        attributes_open.set(attributes_for.is_some());
    }

    let attributes = attributes_for.and_then(|index| filter.enemies.get(index).map(|enemy| (index, enemy.attributes.clone())));

    if attributes_for.is_some() && !attributes_open() {
        state.attributes_for.set(None);
    }

    rsx! {
        Popup { title: "Advanced Stage Filter", open: state.filter_open, width: 720.0,
            div { class: "filter-content",
                div { class: "section",
                    div { class: "section-title", "Name" }
                    div { class: "rule" }
                    {name_field("Category:", &filter.category_name, state, |next, value| next.category_name = value)}
                    {name_field("Map:", &filter.map_name, state, |next, value| next.map_name = value)}
                    {name_field("Stage:", &filter.stage_name, state, |next, value| next.stage_name = value)}
                }
                div { class: "section",
                    div { class: "section-title", "General Rules" }
                    div { class: "rule" }
                    {flags(&GENERAL_RULES, &filter, names, state)}
                }
                div { class: "section",
                    div { class: "section-title", "Special Map Rules" }
                    div { class: "rule" }
                    {flags(&SPECIAL_RULES, &filter, names, state)}
                }
                div { class: "section",
                    div { class: "section-title", "Score Bonuses" }
                    div { class: "rule" }
                    {flags(&SCORE_BONUSES, &filter, names, state)}
                }
                div { class: "section",
                    div { class: "section-title", "Entry Cost" }
                    div { class: "rule" }
                    {cost_fields(&filter, state)}
                }
                div { class: "section",
                    div { class: "section-title", "Stats" }
                    div { class: "rule" }
                    {ranges(&STATS, &filter, state)}
                }
                div { class: "section",
                    div { class: "section-title", "Restrictions" }
                    div { class: "rule" }
                    {ranges(&RESTRICTIONS, &filter, state)}
                }
                div { class: "section",
                    div { class: "section-title", "IDs & Audio" }
                    div { class: "rule" }
                    {ranges(&IDS_AUDIO, &filter, state)}
                }
                div { class: "section",
                    div { class: "section-title", "Battleground" }
                    div { class: "rule" }
                    for (index, enemy) in filter.enemies.iter().enumerate() {
                        {enemy_card(index, enemy, state)}
                    }
                    button { class: "btn btn-primary add-btn", onclick: move |_| edit(state, |next| next.enemies.push(EnemyFilter::default())), "+ Add New Enemy" }
                }
                div { class: "section",
                    div { class: "section-title", "Fixed Lineup Cats" }
                    div { class: "rule" }
                    for (index, cat) in filter.lineup_cats.iter().enumerate() {
                        {lineup_card(index, cat, state)}
                    }
                    button { class: "btn btn-primary add-btn", onclick: move |_| edit(state, |next| next.lineup_cats.push(LineupFilter::default())), "+ Add Lineup Cat" }
                }
                div { class: "section",
                    div { class: "section-title", "Treasures" }
                    div { class: "rule" }
                    for (index, treasure) in filter.treasures.iter().enumerate() {
                        {treasure_card(index, treasure, state)}
                    }
                    button { class: "btn btn-primary add-btn", onclick: move |_| edit(state, |next| next.treasures.push(TreasureFilter::default())), "+ Add New Treasure" }
                }
                div { class: "section",
                    div { class: "section-title", "Materials" }
                    div { class: "rule" }
                    for (index, material) in filter.materials.iter().enumerate() {
                        {material_card(index, material, state)}
                    }
                    button { class: "btn btn-primary add-btn", onclick: move |_| edit(state, |next| next.materials.push(MaterialFilter::default())), "+ Add New Material" }
                }
                div { class: "filter-clear",
                    button { class: "btn btn-danger", onclick: move |_| edit(state, |next| *next = StageFilterState { is_open: next.is_open, ..StageFilterState::default() }), "Clear Filter" }
                }
            }
        }
        if let Some((index, attributes)) = attributes {
            Popup { title: "Battleground Attributes", open: attributes_open,
                Content {
                    available: available(),
                    globals: globals.clone(),
                    filter: Edited(attributes),
                    magnification_hint: "Stage",
                    on_change: move |next: EnemyFilterState| edit(state, |held| { if let Some(enemy) = held.enemies.get_mut(index) { enemy.attributes = next; } }),
                }
            }
        }
    }
}
