use std::collections::BTreeMap;

pub fn get_condition_flag(flags: &BTreeMap<i32, bool>, id: i32) -> Option<bool> {
    flags.get(&id).copied()
}
