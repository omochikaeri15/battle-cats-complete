use crate::Fault;

use super::AppContext;

pub fn has_shield_pierce_chance(ctx: &AppContext, team: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(team, slot, 0x83bd4))? != 0)
}
