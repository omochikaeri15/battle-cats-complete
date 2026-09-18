use crate::fault::Fault;

use super::AppContext;

pub fn get_took_damage(ctx: &AppContext, team: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(team, slot, 0x83b80))? != 0)
}
