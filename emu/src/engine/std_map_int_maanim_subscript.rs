use std::collections::BTreeMap;

use super::Maanim;

pub fn std_map_int_maanim_subscript<'a>(
    map: &'a mut BTreeMap<i32, Maanim>,
    key: &i32,
) -> &'a mut Maanim {
    map.entry(*key).or_default()
}
