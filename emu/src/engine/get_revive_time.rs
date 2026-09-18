use crate::Fault;

use super::{read_flag, AppContext, Entity};

pub fn get_revive_time(ctx: &AppContext, faction: i32, slot: i32) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        return Ok(0);
    }

    ctx.i32_at(AppContext::entity_field(faction, slot, Entity::REVIVE_TIME))
}
