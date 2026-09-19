use std::collections::BTreeMap;

use crate::Fault;

pub fn get_star_multiplier(
    store: &BTreeMap<i32, Vec<i32>>,
    map_id: i32,
    star: i32,
) -> Result<i32, Fault> {
    let Some(multipliers) = store.get(&map_id) else {
        return Ok(star.wrapping_mul(0x32).wrapping_add(0x64));
    };

    multipliers
        .get(star as usize)
        .copied()
        .ok_or(Fault::IndexOutOfRange {
            site: "get_star_multiplier",
            index: star as i64,
            limit: multipliers.len() as i64,
        })
}
