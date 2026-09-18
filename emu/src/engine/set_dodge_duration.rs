use crate::Fault;

use super::{AppContext, Entity};

pub fn set_dodge_duration(ctx: &mut AppContext, faction: i32, slot: i32, value: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::entity_field(faction, slot, Entity::DODGE_DURATION), value)
}
