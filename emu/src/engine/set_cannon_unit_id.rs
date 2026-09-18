use crate::Fault;

use super::{AppContext, Entity};

pub fn set_cannon_unit_id(ctx: &mut AppContext, faction: i32, unit_id: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::entity_field(faction, 0, Entity::CANNON_UNIT_ID), unit_id)
}
