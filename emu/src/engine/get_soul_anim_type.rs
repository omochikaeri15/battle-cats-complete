use crate::Fault;

use super::{AppContext, Entity};

pub fn get_soul_anim_type(ctx: &AppContext, faction: i32, slot: i32) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::entity_field(
        faction,
        slot,
        Entity::SOUL_ANIM_TYPE,
    ))
}
