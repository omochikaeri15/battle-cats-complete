use crate::Fault;

use super::AppContext;

pub fn get_surge_immune(ctx: &AppContext, team: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(team, slot, 0x83bc4))? != 0)
}
