use std::collections::BTreeMap;

pub fn std_map_string_string_count(map: &BTreeMap<Vec<u8>, Vec<u8>>, key: &[u8]) -> u64 {
    map.contains_key(key) as u64
}
