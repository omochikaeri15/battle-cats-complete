use crate::Fault;

use super::{AppContext, Entity};

pub fn get_metal_killer_vfx(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(
        faction,
        slot,
        Entity::METAL_KILLER_VFX,
    ))? != 0)
}
