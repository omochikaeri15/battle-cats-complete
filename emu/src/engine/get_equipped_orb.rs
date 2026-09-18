use crate::Fault;

use super::{get_equipped_orbs, AppContext};

pub fn get_equipped_orb(ctx: &AppContext, unit_id: i32, slot: i32) -> Result<i32, Fault> {
    let equipped = get_equipped_orbs(ctx, unit_id)?;

    if !equipped.contains_key(&slot) {
        return Ok(-1);
    }

    equipped.get(&slot).copied().ok_or(Fault::KeyNotFound { site: "get_equipped_orb", key: slot as i64 })
}
