use crate::Fault;

use super::{AppContext, Entity};

pub fn add_pos_x(ctx: &mut AppContext, faction: i32, slot: i32, dx: i32) -> Result<(), Fault> {
    let field = AppContext::entity_field(faction, slot, Entity::POS_X);
    let current = ctx.i32_at(field)?;

    ctx.set_i32_at(field, current.wrapping_add(dx))
}
