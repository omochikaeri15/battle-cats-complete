use crate::Fault;

use super::{AppContext, Entity};

pub fn get_spawn_anim_flag(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(faction, slot, Entity::SPAWN_ANIM_FLAG))? != 0)
}
