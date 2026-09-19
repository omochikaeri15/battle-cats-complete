use super::EventItemStore;

pub fn advance_point_decay(store: &mut EventItemStore) {
    if store.progress_cap != -1 && store.progress_cap <= store.progress {
        return;
    }

    store.progress = store.progress.wrapping_add(1);
}
