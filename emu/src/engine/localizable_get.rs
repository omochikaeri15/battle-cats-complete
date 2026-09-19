use std::collections::BTreeMap;

use super::std_map_string_string_find;

pub fn localizable_get(table: &BTreeMap<Vec<u8>, Vec<u8>>, key: &[u8]) -> Vec<u8> {
    std_map_string_string_find(table, key).cloned().unwrap_or_else(|| key.to_vec())
}
