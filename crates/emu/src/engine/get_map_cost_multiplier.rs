use std::collections::BTreeMap;

pub fn get_map_cost_multiplier(store: &BTreeMap<i32, i32>, map_id: i32) -> i32 {
    if !store.contains_key(&map_id) {
        return 0;
    }

    store.get(&map_id).copied().unwrap_or(0)
}
