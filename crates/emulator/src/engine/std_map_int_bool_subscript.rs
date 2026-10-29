use std::collections::BTreeMap;

pub fn std_map_int_bool_subscript<'a>(map: &'a mut BTreeMap<i32, bool>, key: &i32) -> &'a mut bool {
    map.entry(*key).or_default()
}
