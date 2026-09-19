use crate::Fault;

use super::{AppContext, Entity, read_flag};

pub fn has_eva_killer(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return Ok(false);
    }

    Ok(ctx.i32_at(AppContext::entity_field(faction, slot, Entity::EVA_KILLER))? != 0)
}
