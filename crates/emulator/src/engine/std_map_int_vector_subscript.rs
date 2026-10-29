use std::collections::BTreeMap;

pub fn std_map_int_vector_subscript<'a>(
    map: &'a mut BTreeMap<i32, Vec<i32>>,
    key: &i32,
) -> &'a mut Vec<i32> {
    map.entry(*key).or_default()
}
