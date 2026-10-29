use crate::Fault;

use super::{AppContext, Entity};

pub fn start_immune_vfx(ctx: &mut AppContext, faction: i32, slot: i32) -> Result<(), Fault> {
    ctx.set_i32_at(
        AppContext::entity_field(faction, slot, Entity::IMMUNE_VFX_FRAME),
        0,
    )?;
    ctx.set_i32_at(
        AppContext::entity_field(faction, slot, Entity::IMMUNE_VFX_ACTIVE),
        1,
    )
}
