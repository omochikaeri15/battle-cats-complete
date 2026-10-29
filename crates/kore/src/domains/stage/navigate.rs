use std::collections::HashSet;

use nyanko::chapter::{Category, map::{fixed_invasion_target, InvasionTarget}};

use super::{GlobalMapId, GlobalStageId, Map, Stage, StageRegistry};

pub fn get_categories(registry: &StageRegistry) -> Vec<Category> {
    let unique_categories: HashSet<Category> = registry.maps.keys()
        .map(|id| id.category.clone())
        .collect();

    unique_categories.into_iter().collect()
}

pub fn get_maps<'a>(registry: &'a StageRegistry, category: &Category) -> Vec<&'a Map> {
    let mut maps: Vec<&Map> = registry.maps.iter()
        .filter(|(id, _)| id.category == *category)
        .map(|(_, m)| m)
        .collect();

    maps.sort_by_key(|m| m.map_id);
    maps
}

pub fn get_stages<'a>(registry: &'a StageRegistry, map_id: &GlobalMapId) -> Vec<&'a Stage> {
    let Some(map) = registry.maps.get(map_id) else { return Vec::new(); };

    let mut stages: Vec<&Stage> = map.stages.iter()
        .filter_map(|&stage_id| {
            let stage_key = GlobalStageId {
                category: map_id.category.clone(),
                map: map_id.map,
                stage: stage_id,
            };
            registry.stages.get(&stage_key)
        })
        .collect();

    stages.sort_by_key(|s| s.stage_id);
    stages
}
const STORY_MOON_SLOT: i32 = 47;
const INVASION_MARK: &str = "_Invasion";

pub fn map_screen_slot(global_map_id: i32, stage_id: i32) -> i32 {
    match (global_map_id, stage_id) {
        (3001, 49) | (3002, 50) => STORY_MOON_SLOT,
        _ => stage_id,
    }
}

pub fn is_invasion(registry: &StageRegistry, stage: &GlobalStageId) -> bool {
    registry.grounds.get(stage).is_some_and(|file| file.contains(INVASION_MARK))
}

pub fn is_invader(registry: &StageRegistry, map_id: &GlobalMapId) -> bool {
    let Some(global) = registry.addresses.get(map_id).and_then(|address| address.global) else {
        return false;
    };

    fixed_invasion_target(global).is_some() || registry.maps.values().any(|map| map.ex_invasion == Some(global))
}

pub fn invaded_stage(registry: &StageRegistry, stage: &GlobalStageId) -> Option<InvasionTarget> {
    let map = GlobalMapId { category: stage.category.clone(), map: stage.map };
    let global = registry.addresses.get(&map)?.global?;

    fixed_invasion_target(global).or_else(|| {
        let source = registry.maps.values().find(|source| source.ex_invasion == Some(global))?;
        let key = GlobalMapId { category: source.category.clone(), map: source.map_id };

        Some(InvasionTarget { map_id: registry.addresses.get(&key)?.global?, stage: stage.stage, crown: 0 })
    })
}

pub fn invadable_stages<'a>(registry: &'a StageRegistry, map_id: &GlobalMapId) -> Vec<&'a Stage> {
    get_stages(registry, map_id)
        .into_iter()
        .filter(|stage| {
            let key = GlobalStageId { category: map_id.category.clone(), map: map_id.map, stage: stage.stage_id };

            !is_invasion(registry, &key)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // The registry numbers the chapter 2 and 3 moons by their stage file (49, 50); the
    // game's map screen enters every moon as slot 47 and shifts to the file itself.
    #[test]
    fn empire_moons_enter_as_slot_47() {
        assert_eq!(map_screen_slot(3000, 47), 47);
        assert_eq!(map_screen_slot(3001, 49), 47);
        assert_eq!(map_screen_slot(3002, 50), 47);
        assert_eq!(map_screen_slot(3002, 46), 46);
        assert_eq!(map_screen_slot(3005, 50), 50);
    }
}
