use crate::Fault;

use super::AppContext;

pub fn get_boss_wave_immune(ctx: &AppContext, team: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(team, slot, 0x83ac8))? != 0)
}
