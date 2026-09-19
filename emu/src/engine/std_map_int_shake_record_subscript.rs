use std::collections::BTreeMap;

use super::ShakeRecord;

pub fn std_map_int_shake_record_subscript<'a>(
    map: &'a mut BTreeMap<i32, ShakeRecord>,
    key: &i32,
) -> &'a mut ShakeRecord {
    map.entry(*key).or_default()
}
