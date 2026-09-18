use crate::Fault;

use super::AppContext;

pub fn get_wave_block(ctx: &AppContext, team: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(team, slot, 0x83a54))? != 0)
}
