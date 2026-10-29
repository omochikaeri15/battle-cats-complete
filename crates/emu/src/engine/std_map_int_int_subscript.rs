use std::collections::BTreeMap;

pub fn std_map_int_int_subscript<'a>(map: &'a mut BTreeMap<i32, i32>, key: &i32) -> &'a mut i32 {
    map.entry(*key).or_default()
}
