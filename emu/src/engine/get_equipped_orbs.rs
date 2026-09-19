use std::collections::BTreeMap;

use crate::Fault;

use super::{AppContext, std_map_int_int_from_list_2, std_map_int_int_insert_range};

pub fn get_equipped_orbs(ctx: &AppContext, unit_id: i32) -> Result<BTreeMap<i32, i32>, Fault> {
    if !ctx.equipped_orbs.contains_key(&unit_id) {
        return Ok(std_map_int_int_from_list_2(&[(0, -1)]));
    }

    let slots = ctx.equipped_orbs.get(&unit_id).ok_or(Fault::KeyNotFound {
        site: "get_equipped_orbs",
        key: unit_id as i64,
    })?;
    let mut equipped = BTreeMap::new();

    std_map_int_int_insert_range(&mut equipped, slots);

    Ok(equipped)
}
