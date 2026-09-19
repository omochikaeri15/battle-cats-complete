use std::collections::BTreeMap;

use super::Mamodel;

pub fn std_map_int_mamodel_subscript<'a>(
    map: &'a mut BTreeMap<i32, Mamodel>,
    key: &i32,
) -> &'a mut Mamodel {
    map.entry(*key).or_default()
}
