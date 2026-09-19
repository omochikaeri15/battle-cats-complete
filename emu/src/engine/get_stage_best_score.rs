use super::{EventItemStore, save_data_get_best};

pub fn get_stage_best_score(store: &EventItemStore, stage: i32) -> i32 {
    let Some(point_id) = store.stage_points.get(&store.stage_key) else {
        return 0;
    };

    store.records.get(point_id).map_or(0, |record| {
        save_data_get_best(record, store.stage_key, stage)
    })
}
