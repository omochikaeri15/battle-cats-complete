use super::SpecialRuleStore;

pub fn set_fever_fade_out(store: &mut SpecialRuleStore) {
    store.fade_reverse = 1;
}
