use crate::Fault;

use super::{AppContext, Entity};

pub fn set_frame_damage(
    ctx: &mut AppContext,
    faction: i32,
    slot: i32,
    value: i32,
) -> Result<(), Fault> {
    ctx.set_i32_at(
        AppContext::entity_field(faction, slot, Entity::FRAME_DAMAGE),
        value,
    )?;

    let took_damage = value != 0;

    ctx.set_i32_at(
        AppContext::entity_field(faction, slot, Entity::TOOK_DAMAGE),
        took_damage as i32,
    )
}
