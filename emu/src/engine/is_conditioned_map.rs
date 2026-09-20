use super::MapOption;

pub fn is_conditioned_map(store: &MapOption, map_id: i32) -> bool {
    store.conditioned_maps.contains(&map_id)
}
