use crate::Fault;

use super::{AppContext, Entity};

pub fn set_pos_x(ctx: &mut AppContext, faction: i32, slot: i32, x: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::entity_field(faction, slot, Entity::POS_X), x)
}
