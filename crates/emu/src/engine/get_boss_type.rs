use crate::Fault;

use super::{AppContext, Entity, read_flag};

pub fn get_boss_type(ctx: &AppContext, faction: i32, slot: i32) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        return Ok(0);
    }

    ctx.i32_at(AppContext::entity_field(faction, slot, Entity::BOSS_TYPE))
}
