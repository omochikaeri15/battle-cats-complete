use std::collections::BTreeMap;

use crate::Fault;

pub fn get_crown_multiplier(
    store: &BTreeMap<i32, Vec<i32>>,
    map_id: i32,
    crown: i32,
) -> Result<i32, Fault> {
    let Some(multipliers) = store.get(&map_id) else {
        return Ok(crown.wrapping_mul(0x32).wrapping_add(0x64));
    };

    multipliers
        .get(crown as usize)
        .copied()
        .ok_or(Fault::index_out_of_range(crown as i64, multipliers.len() as i64))
}
