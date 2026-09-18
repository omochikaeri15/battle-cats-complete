use crate::Fault;

use super::{AppContext, Entity};

pub fn get_cannon_unit_id(ctx: &AppContext, faction: i32) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::entity_field(faction, 0, Entity::CANNON_UNIT_ID))
}
