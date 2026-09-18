use crate::Fault;

use super::AppContext;

pub fn add_warp_timer(ctx: &mut AppContext, team: i32, slot: i32, delta: i32) -> Result<(), Fault> {
    let field = AppContext::entity_field(team, slot, 0x83b24);
    let current = ctx.i32_at(field)?;

    ctx.set_i32_at(field, current.wrapping_add(delta))
}
