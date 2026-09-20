use std::collections::BTreeMap;

use crate::Fault;

use super::{std_map_string_string_count, std_map_string_string_find_equal, std_stod};

pub fn get_setting_double(
    store: &BTreeMap<Vec<u8>, Vec<u8>>,
    key: &[u8],
    fallback: f64,
) -> Result<f64, Fault> {
    if std_map_string_string_count(store, key) == 0 {
        return Ok(fallback);
    }

    let value = std_map_string_string_find_equal(store, key).ok_or(Fault::key_not_found(0))?;

    if value.is_empty() {
        return Ok(fallback);
    }

    let value = std_map_string_string_find_equal(store, key).ok_or(Fault::key_not_found(0))?;

    std_stod(value, None)
}
