use std::collections::HashSet;

use nyanko::chapter::Category;

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

pub fn map_screen_slot(global_map_id: i32, stage_id: i32) -> i32 {
    match (global_map_id, stage_id) {
        (3001, 49) | (3002, 50) => STORY_MOON_SLOT,
        _ => stage_id,
    }
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
