use super::SpecialRuleStore;

pub fn fever_clear_state(store: &mut SpecialRuleStore) {
    store.fever_frame = 0x64;
    store.fade_reverse = 0;
    store.gauge_fill = -1;
}
