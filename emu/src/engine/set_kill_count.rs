use crate::Fault;

use super::AppContext;

pub fn set_kill_count(ctx: &mut AppContext, team: i32, slot: i32, value: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::entity_field(team, slot, 0x83cbc), value)
}
