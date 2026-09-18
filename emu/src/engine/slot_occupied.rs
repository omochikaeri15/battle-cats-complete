use crate::Fault;

use super::{AppContext, Entity};

pub fn slot_occupied(ctx: &AppContext, faction: i32, slot: i32) -> Result<i32, Fault> {
    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::SLOT_KIND))? == 0 {
        return Ok(0);
    }

    let is_unit = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::SLOT_KIND))? != 1;

    Ok(is_unit as i32 + 1)
}
