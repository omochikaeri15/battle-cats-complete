use crate::Fault;

use super::AppContext;

pub fn add_orb_dodge_timer(ctx: &mut AppContext, faction: i32, slot: i32, delta: i32) -> Result<(), Fault> {
    let field = AppContext::entity_field(faction, slot, 0x83ca0);
    let current = ctx.i32_at(field)?;

    ctx.set_i32_at(field, current.wrapping_add(delta))
}
