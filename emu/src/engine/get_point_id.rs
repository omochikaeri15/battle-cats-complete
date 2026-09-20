use super::EventItemStore;

pub fn get_point_id(store: &EventItemStore) -> i32 {
    if store.stage_key == -1 {
        return 0;
    }

    store
        .point_id_by_map
        .get(&store.stage_key)
        .copied()
        .unwrap_or(0)
}
