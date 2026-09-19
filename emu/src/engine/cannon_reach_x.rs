use crate::{Fault, operation};

use super::{AppContext, Entity};

pub fn cannon_reach_x(ctx: &AppContext, faction: i32) -> Result<i32, Fault> {
    let screen_x = operation::div_10(
        ctx.i32_at(AppContext::entity_field(faction, 0, Entity::POS_X))?
            .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?),
    );

    Ok(if faction == 0 {
        screen_x.wrapping_add(0x1b)
    } else {
        screen_x.wrapping_add(-0x1b)
    })
}
