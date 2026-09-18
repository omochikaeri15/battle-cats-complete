use crate::Fault;

use super::AppContext;

pub fn get_weaken_resist_pct(ctx: &AppContext, team: i32, slot: i32) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::entity_field(team, slot, 0x83b5c))
}
