use crate::Fault;

use super::{AppContext, ATTACK_ABILITY_FIELDS};

pub fn set_attack_abilities(ctx: &mut AppContext, faction: i32, slot: i32, attack: i32, value: i32) -> Result<(), Fault> {
    let field = *ATTACK_ABILITY_FIELDS
        .get(attack as usize)
        .ok_or(Fault::IndexOutOfRange { site: "set_attack_abilities", index: attack as i64, limit: 3 })?;

    ctx.set_i32_at(AppContext::entity_field(faction, slot, 0).wrapping_add((field as usize).wrapping_mul(4)), value)
}
