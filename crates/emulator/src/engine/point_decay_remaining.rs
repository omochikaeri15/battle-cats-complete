use super::EventItemStore;

pub fn point_decay_remaining(store: &EventItemStore) -> i32 {
    store.progress_cap.wrapping_sub(store.progress)
}
