use crate::Fault;

use super::{AppContext, ATTACK_ABILITY_FIELDS};

pub fn has_attack_abilities(ctx: &AppContext, faction: i32, slot: i32, attack: i32) -> Result<bool, Fault> {
    let field = *ATTACK_ABILITY_FIELDS
        .get(attack as usize)
        .ok_or(Fault::IndexOutOfRange { site: "has_attack_abilities", index: attack as i64, limit: 3 })?;

    Ok(ctx.i32_at(AppContext::entity_field(faction, slot, 0).wrapping_add((field as usize).wrapping_mul(4)))? != 0)
}
