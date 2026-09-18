use crate::Fault;

use super::{AppContext, Entity};

pub fn get_cannon_hit_stamp(ctx: &AppContext, faction: i32, slot: i32) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::entity_field(faction, slot, Entity::CANNON_HIT_STAMP))
}
