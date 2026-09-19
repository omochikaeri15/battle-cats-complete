use crate::Fault;

use super::EventItemStore;

pub fn event_items_count(store: &EventItemStore, item: i32) -> Result<i32, Fault> {
    store.counts.get(&item).copied().ok_or(Fault::KeyNotFound { site: "event_items_count", key: item as i64 })
}
