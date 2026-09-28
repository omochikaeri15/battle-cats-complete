use std::collections::BTreeMap;

pub fn get_condition_flag(flags: &mut BTreeMap<i32, bool>, id: i32) -> Option<bool> {
    if flags.is_empty() {
        return None;
    }

    Some(*flags.entry(id).or_insert(false))
}
