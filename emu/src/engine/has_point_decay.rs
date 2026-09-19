use super::EventItemStore;

pub fn has_point_decay(store: &EventItemStore) -> bool {
    store.progress_cap != -1
}
