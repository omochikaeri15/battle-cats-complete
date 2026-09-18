use crate::Fault;

use super::{AppContext, Entity};

pub fn add_barrier_fx_frame(ctx: &mut AppContext, faction: i32, slot: i32, delta: i32) -> Result<(), Fault> {
    let field = AppContext::entity_field(faction, slot, Entity::BARRIER_FX_FRAME);
    let current = ctx.i32_at(field)?;

    ctx.set_i32_at(field, current.wrapping_add(delta))
}
