use crate::Fault;

use super::{AppContext, Entity};

pub fn get_barrier_vfx_active(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(
        faction,
        slot,
        Entity::BARRIER_VFX_ACTIVE,
    ))? != 0)
}
