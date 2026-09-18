use crate::Fault;

use super::AppContext;

pub fn get_drain_chance(ctx: &AppContext, team: i32, slot: i32) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::entity_field(team, slot, 0x83cd0))
}
