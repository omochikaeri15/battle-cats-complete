use crate::Fault;

use super::AppContext;

pub fn add_drain_pct(ctx: &mut AppContext, faction: i32, slot: i32, delta: i32) -> Result<(), Fault> {
    let field = AppContext::entity_field(faction, slot, 0x83cd8);
    let current = ctx.i32_at(field)?;

    ctx.set_i32_at(field, current.wrapping_add(delta))
}
