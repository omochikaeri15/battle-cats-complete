use std::collections::BTreeMap;

pub fn std_map_string_string_find_equal<'a>(
    map: &'a BTreeMap<Vec<u8>, Vec<u8>>,
    key: &[u8],
) -> Option<&'a Vec<u8>> {
    map.get(key)
}
