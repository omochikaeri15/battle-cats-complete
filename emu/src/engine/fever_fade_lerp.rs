use crate::operation;

use super::SpecialRuleStore;

pub fn fever_fade_lerp(store: &SpecialRuleStore, value: i32) -> i32 {
    operation::div_5(5i32.wrapping_sub(store.fade_phase).wrapping_mul(value))
}
