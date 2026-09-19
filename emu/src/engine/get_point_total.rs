use super::EventItemStore;

pub fn get_point_total(store: &EventItemStore) -> i32 {
    store
        .stage_points
        .get(&store.stage_key)
        .and_then(|point_id| store.records.get(point_id))
        .map_or(0, |record| record.total)
}
