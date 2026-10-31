use std::collections::BTreeSet;

use emulator::runtime::Setup;
use kokoro::domains::cat::scanner::CatEntry;
use kokoro::domains::stage::{GlobalStageId, StageRegistry, navigate};

use super::Picked;

const FORM_TAGS: [char; 4] = ['f', 'c', 's', 'u'];
const WARM_POOLS: [&str; 2] = ["caf", "roster:battle"];
const BATTLE_STEMS: [&str; 74] = [
    "000_a",
    "000_g",
    "FeverEffect",
    "battle_demonsoul_00",
    "battle_demonsoul_01",
    "battle_entry_",
    "battle_soul_",
    "boss_welcome",
    "bubble",
    "critical",
    "demonbattle_000",
    "download",
    "equipment_attribute",
    "equipment_effect",
    "equipment_grade",
    "equipment_shadow",
    "gatyaitem_000_f",
    "gatyaitemD_",
    "img001",
    "img002",
    "img003",
    "img004",
    "img005",
    "img006",
    "img007",
    "img008",
    "img009",
    "img010",
    "img015",
    "img024",
    "img039",
    "img040",
    "img041",
    "img042",
    "img043",
    "img060_02",
    "img100",
    "img101",
    "invoke_equipment",
    "mapicon",
    "mapsnALL_all",
    "nyankoCastle_",
    "set_enemy001_zombie",
    "skill",
    "uni.",
    "FeverEffect.",
    "castle_ec",
    "castle_rc",
    "castle_wc",
    "img005_1",
    "img008_Labyrinth",
    "img008_nekoDojo",
    "img009_Labyrinth_001",
    "img009_Labyrinth_result",
    "img009_nekoDojo_001",
    "img009_nekoDojo_result",
    "img101_nekoDojo",
    "img004_DOJO_",
    "skill_attack_invalid",
    "skill_curse",
    "skill_down",
    "skill_effect_invalid",
    "skill_explosion",
    "skill_guard_e",
    "skill_metal_strong",
    "skill_percentage_attack",
    "skill_shield",
    "skill_slow",
    "skill_stop",
    "skill_up",
    "skill_volcano",
    "skill_warp",
    "skill_wave_attack",
    "skill_zombie_strong",
];

pub(crate) struct Warmth {
    pub(crate) prefixes: Vec<Box<str>>,
    pub(crate) pools: Vec<Box<str>>,
}

fn unit_prefixes(prefixes: &mut BTreeSet<String>, cat: Option<&CatEntry>, id: i32, form: i32) {
    let Ok(id) = u32::try_from(id) else {
        return;
    };
    let form = usize::try_from(form).unwrap_or(0).min(FORM_TAGS.len() - 1);
    let tag = FORM_TAGS[form];

    prefixes.insert(format!("{id:03}_{tag}"));
    prefixes.insert(format!("uni{id:03}_{tag}"));

    if let Some((norm, evol)) = cat.and_then(|cat| cat.egg_ids) {
        let art = if form == 0 { norm } else { evol };

        if form <= 1 && art >= 0 {
            prefixes.insert(format!("{art:03}_m"));
            prefixes.insert(format!("uni{art:03}_m"));
        }
    }
}

fn stage_prefixes(prefixes: &mut BTreeSet<String>, registry: &StageRegistry, key: &GlobalStageId) {
    let Some(stage) = registry.stages.get(key) else {
        return;
    };

    for enemy in &stage.enemies {
        prefixes.insert(format!("{:03}_e", enemy.enemy_id));
        prefixes.insert(format!("i{:03}_e", enemy.enemy_id));
    }

    prefixes.insert(format!("ec{:03}", stage.anim_base_id));
    prefixes.insert(format!("wc{:03}", stage.anim_base_id));
    prefixes.insert(format!("sc{:03}", stage.anim_base_id));
    prefixes.insert(format!("bg{:03}", stage.background_id));
    prefixes.insert(format!("bg{:02}", stage.background_id));
    prefixes.insert(format!("snd{:03}.", stage.init_track));

    if let Ok(boss) = u32::try_from(stage.boss_track) {
        prefixes.insert(format!("snd{boss:03}."));
    }

    if let Ok(base) = u32::try_from(stage.base_id) {
        prefixes.insert(format!("rc{base:03}"));
    }
}

pub(crate) fn collect(setup: &Setup, registry: &StageRegistry, picked: &Picked, cats: &[CatEntry]) -> Warmth {
    let mut prefixes: BTreeSet<String> = BATTLE_STEMS.iter().map(|stem| (*stem).to_owned()).collect();

    for unit in &setup.lineup {
        let cat = u32::try_from(unit.unit).ok().and_then(|id| cats.iter().find(|cat| cat.id == id));

        unit_prefixes(&mut prefixes, cat, unit.unit, unit.form);
    }

    stage_prefixes(&mut prefixes, registry, &picked.stage);

    if let Some(target) = navigate::invaded_stage(registry, &picked.stage) {
        for (key, stage) in &registry.stages {
            let global = registry.addresses.get(&kokoro::domains::stage::GlobalMapId { category: key.category.clone(), map: key.map }).and_then(|address| address.global);

            if global == Some(target.map_id) && stage.stage_id == target.stage {
                stage_prefixes(&mut prefixes, registry, key);
            }
        }
    }


    Warmth { prefixes: prefixes.into_iter().map(Box::from).collect(), pools: WARM_POOLS.iter().map(|pool| Box::from(*pool)).collect() }
}
