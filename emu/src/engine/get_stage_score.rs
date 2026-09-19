use super::EventItemStore;

pub fn get_stage_score(store: &EventItemStore) -> i32 {
    store.total
}
