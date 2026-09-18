use crate::fault::Fault;

use super::AppContext;

pub fn get_standing_range(ctx: &AppContext, team: i32, slot: i32) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::entity_field(team, slot, 0x8392c))
}
