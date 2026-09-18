use crate::fault::Fault;

use super::AppContext;

pub fn set_pos_x(ctx: &mut AppContext, team: i32, slot: i32, x: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::entity_field(team, slot, 0x83904), x)
}
