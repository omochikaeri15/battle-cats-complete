use super::EventItemStore;

pub fn point_decay_full(store: &EventItemStore) -> bool {
    store.progress_cap != -1 && store.progress_cap <= store.progress
}
