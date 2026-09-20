use crate::Fault;

use super::{ATTACK_FORESWING_FIELDS, AppContext};

pub fn get_attack_foreswing(
    ctx: &AppContext,
    faction: i32,
    slot: i32,
    attack: i32,
) -> Result<i32, Fault> {
    let field = *ATTACK_FORESWING_FIELDS
        .get(attack as usize)
        .ok_or(Fault::index_out_of_range(attack as i64, 3))?;

    ctx.i32_at(
        AppContext::entity_field(faction, slot, 0).wrapping_add((field as usize).wrapping_mul(4)),
    )
}
