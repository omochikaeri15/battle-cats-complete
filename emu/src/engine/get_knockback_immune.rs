use crate::Fault;

use super::{AppContext, Entity};

pub fn get_knockback_immune(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(faction, slot, Entity::KNOCKBACK_IMMUNE))? != 0)
}
