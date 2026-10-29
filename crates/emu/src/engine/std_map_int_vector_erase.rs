use std::collections::BTreeMap;

pub fn std_map_int_vector_erase(map: &mut BTreeMap<i32, Vec<i32>>, key: &i32) {
    map.remove(key);
}
