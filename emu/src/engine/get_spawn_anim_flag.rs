use crate::Fault;

use super::AppContext;

pub fn get_spawn_anim_flag(ctx: &AppContext, team: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(team, slot, 0x83af8))? != 0)
}
