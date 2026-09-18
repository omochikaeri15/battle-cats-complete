use crate::Fault;

use super::AppContext;

pub fn add_shield_fx_frame(ctx: &mut AppContext, faction: i32, slot: i32, delta: i32) -> Result<(), Fault> {
    let field = AppContext::entity_field(faction, slot, 0x83bfc);
    let current = ctx.i32_at(field)?;

    ctx.set_i32_at(field, current.wrapping_add(delta))
}
