use super::{EventItemStore, save_data_set_best};

pub fn set_stage_best_score(
    store: &mut EventItemStore,
    point_id: i32,
    key: i32,
    stage: i32,
    value: i32,
) {
    save_data_set_best(
        store.records.entry(point_id).or_default(),
        key,
        stage,
        value,
    );
}
