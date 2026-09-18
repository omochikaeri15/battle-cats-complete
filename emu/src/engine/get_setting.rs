use std::collections::BTreeMap;

use crate::Fault;

use super::{std_map_string_string_count, std_map_string_string_find_equal, std_stoi};

pub fn get_setting(store: &BTreeMap<Vec<u8>, Vec<u8>>, key: &[u8], fallback: i32) -> Result<i32, Fault> {
    if std_map_string_string_count(store, key) == 0 {
        return Ok(fallback);
    }

    let value = std_map_string_string_find_equal(store, key).ok_or(Fault::KeyNotFound { site: "get_setting", key: 0 })?;

    if value.is_empty() {
        return Ok(fallback);
    }

    let value = std_map_string_string_find_equal(store, key).ok_or(Fault::KeyNotFound { site: "get_setting", key: 0 })?;

    std_stoi(value, None, 0xa)
}
