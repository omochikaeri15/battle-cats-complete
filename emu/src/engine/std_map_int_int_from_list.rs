use std::collections::BTreeMap;

pub fn std_map_int_int_from_list(pairs: &[(i32, i32)]) -> BTreeMap<i32, i32> {
    let mut map = BTreeMap::new();

    for (key, value) in pairs {
        map.entry(*key).or_insert(*value);
    }

    map
}
