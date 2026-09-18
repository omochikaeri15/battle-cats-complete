use crate::Fault;

use super::{AppContext, Entity};

pub fn get_dodge_timer(ctx: &AppContext, faction: i32, slot: i32) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::entity_field(faction, slot, Entity::DODGE_TIMER))
}
