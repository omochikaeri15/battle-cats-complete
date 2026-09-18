use crate::Fault;

use super::{AppContext, Entity};

pub fn get_took_damage(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(faction, slot, Entity::TOOK_DAMAGE))? != 0)
}
