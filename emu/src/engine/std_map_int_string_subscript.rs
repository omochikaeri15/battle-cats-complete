use std::collections::BTreeMap;

pub fn std_map_int_string_subscript<'a>(map: &'a mut BTreeMap<i32, Vec<u8>>, key: &i32) -> &'a mut Vec<u8> {
    map.entry(*key).or_default()
}
