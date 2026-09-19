use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub struct EventItemStore {
    pub counts: BTreeMap<i32, i32>,
    pub listed: BTreeSet<i32>,
}

pub fn event_items_has(store: &EventItemStore, item: i32) -> bool {
    store.listed.contains(&item)
}
