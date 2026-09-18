use crate::Fault;

use super::{AppContext, Base};

pub fn add_cannon_shot_id(ctx: &mut AppContext, faction: i32, delta: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::entity_field(faction, 0, Base::CANNON_SHOT_ID), ctx.i32_at(AppContext::entity_field(faction, 0, Base::CANNON_SHOT_ID))?.wrapping_add(delta))
}
