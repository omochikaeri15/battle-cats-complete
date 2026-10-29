use std::collections::BTreeMap;

pub fn std_map_int_map_subscript<'a>(
    map: &'a mut BTreeMap<i32, BTreeMap<i32, bool>>,
    key: &i32,
) -> &'a mut BTreeMap<i32, bool> {
    map.entry(*key).or_default()
}
