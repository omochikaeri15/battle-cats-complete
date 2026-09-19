use crate::Fault;

use super::{AppContext, Entity};

pub fn has_savage_blow_chance(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(
        faction,
        slot,
        Entity::SAVAGE_BLOW_CHANCE,
    ))? != 0)
}
