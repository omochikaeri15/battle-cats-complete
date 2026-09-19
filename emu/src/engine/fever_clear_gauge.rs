use super::SpecialRuleStore;

pub fn fever_clear_gauge(store: &mut SpecialRuleStore) {
    store.point_baseline = 0;
    store.fever_count = 0;
    store.gauge_fill = -1;
}
