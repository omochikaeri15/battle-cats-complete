use crate::Fault;

use super::{AppContext, Entity};

pub fn has_weaken_chance(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(
        faction,
        slot,
        Entity::WEAKEN_CHANCE,
    ))? != 0)
}
