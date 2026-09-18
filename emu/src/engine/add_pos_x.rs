use crate::fault::Fault;

use super::AppContext;

pub fn add_pos_x(ctx: &mut AppContext, team: i32, slot: i32, dx: i32) -> Result<(), Fault> {
    let field = AppContext::entity_field(team, slot, 0x83904);
    let current = ctx.i32_at(field)?;

    ctx.set_i32_at(field, current.wrapping_add(dx))
}
