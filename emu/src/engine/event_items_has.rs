use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub struct PointRuleDetail {
    pub kill_point_base: i32,
    pub bands: BTreeMap<i32, i32>,
}

#[derive(Default)]
pub struct PointRule {
    pub kind: i32,
    pub detail: Option<PointRuleDetail>,
}

#[derive(Default)]
pub struct PointRuleTable {
    pub entries: BTreeMap<i32, PointRule>,
}

#[derive(Default)]
pub struct EventItemStore {
    pub counts: BTreeMap<i32, i32>,
    pub listed: BTreeSet<i32>,
    pub rules: Option<PointRuleTable>,
    pub score_stage: bool,
    pub rule_id: i32,
    pub progress: i32,
    pub progress_cap: i32,
    pub total: i32,
}

pub fn event_items_has(store: &EventItemStore, item: i32) -> bool {
    store.listed.contains(&item)
}
