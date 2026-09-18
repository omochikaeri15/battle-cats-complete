use crate::Fault;

use super::{AppContext, Entity};

pub fn has_drain_chance(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(faction, slot, Entity::DRAIN_CHANCE))? != 0)
}
