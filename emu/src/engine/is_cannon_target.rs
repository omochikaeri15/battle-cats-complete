use crate::Fault;

use super::{AppContext, Entity};

pub fn is_cannon_target(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::STATE))? == 0x13 {
        return Ok(false);
    }

    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::STATE))? == 0x12 {
        return Ok(false);
    }

    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::STATE))? == 0x14 {
        return Ok(false);
    }

    Ok(ctx.i32_at(AppContext::entity_field(faction, slot, Entity::STATE))? != 4)
}
