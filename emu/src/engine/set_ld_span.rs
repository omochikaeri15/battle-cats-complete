use crate::Fault;

use super::{AppContext, ATTACK_LD_SPAN_FIELDS};

pub fn set_ld_span(ctx: &mut AppContext, faction: i32, slot: i32, attack: i32, value: i32) -> Result<(), Fault> {
    let field = *ATTACK_LD_SPAN_FIELDS
        .get(attack as usize)
        .ok_or(Fault::IndexOutOfRange { site: "set_ld_span", index: attack as i64, limit: 3 })?;

    ctx.set_i32_at(AppContext::entity_field(faction, slot, 0x838f8).wrapping_add((field as usize).wrapping_mul(4)), value)
}
