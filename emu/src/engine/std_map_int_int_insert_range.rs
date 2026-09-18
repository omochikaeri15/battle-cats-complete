use std::collections::BTreeMap;

pub fn std_map_int_int_insert_range(map: &mut BTreeMap<i32, i32>, source: &BTreeMap<i32, i32>) {
    for (key, value) in source {
        map.entry(*key).or_insert(*value);
    }
}
