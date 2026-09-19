use crate::{Fault, operation};

use super::{AppContext, CANNON_SHOT_SPACING, Entity, get_base_level};

pub fn cannon_target_in_range(
    ctx: &AppContext,
    faction: i32,
    screen_x: i32,
) -> Result<bool, Fault> {
    if faction == 1 {
        let base_x = operation::div_10(
            ctx.i32_at(AppContext::entity_field(1, 0, Entity::POS_X))?
                .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?),
        );
        let reach = get_base_level(ctx, 1)?.wrapping_mul(CANNON_SHOT_SPACING);

        return Ok(reach.wrapping_add(base_x).wrapping_add(-0x59) > screen_x);
    }

    if faction != 0 {
        return Ok(false);
    }

    let base_x = operation::div_10(
        ctx.i32_at(AppContext::entity_field(0, 0, Entity::POS_X))?
            .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?),
    );
    let reach = get_base_level(ctx, 0)?.wrapping_mul(CANNON_SHOT_SPACING);

    Ok(base_x.wrapping_sub(reach).wrapping_add(0x59) < screen_x)
}
