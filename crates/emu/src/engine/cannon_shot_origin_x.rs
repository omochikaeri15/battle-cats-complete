use crate::Fault;

use super::{AppContext, Entity};

pub fn cannon_shot_origin_x(ctx: &AppContext, faction: i32) -> Result<i32, Fault> {
    let offset: i32 = if faction == 0 { 0x10e } else { -0x10e };

    Ok(offset.wrapping_add(ctx.i32_at(AppContext::entity_field(faction, 0, Entity::POS_X))?))
}
