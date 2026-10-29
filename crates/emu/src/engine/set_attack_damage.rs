use crate::Fault;

use super::{ATTACK_DAMAGE_FIELDS, AppContext};

pub fn set_attack_damage(
    ctx: &mut AppContext,
    faction: i32,
    slot: i32,
    attack: i32,
    value: i32,
) -> Result<(), Fault> {
    let field = *ATTACK_DAMAGE_FIELDS
        .get(attack as usize)
        .ok_or(Fault::index_out_of_range(attack as i64, 3))?;

    ctx.set_i32_at(
        AppContext::entity_field(faction, slot, 0).wrapping_add((field as usize).wrapping_mul(4)),
        value,
    )
}
