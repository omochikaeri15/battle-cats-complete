use super::EventItemStore;

pub fn event_items_count(store: &EventItemStore, item: i32) -> i32 {
    store.records.get(&item).map_or(0, |record| record.total)
}
