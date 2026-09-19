use crate::Fault;

use super::{AppContext, get_current_stage_id, get_scene_id, slot_occupied, stage_has_restriction};

pub fn deploy_limit_reached(ctx: &mut AppContext) -> Result<bool, Fault> {
    let stage_id = get_current_stage_id(ctx)?;

    if !ctx.stage_restrictions.contains_key(&stage_id) {
        stage_has_restriction(ctx, &ctx.stage_restrictions, stage_id)?;

        return Ok(false);
    }

    if !stage_has_restriction(ctx, &ctx.stage_restrictions, stage_id)? {
        return Ok(false);
    }

    if get_scene_id(ctx)? != 0x12c {
        return Ok(false);
    }

    let mut deployed = 0i32;
    let mut slot = 1i32;

    while slot != 51 {
        if slot_occupied(ctx, 0, slot)? != 0 {
            deployed = deployed.wrapping_add(1);
        }

        slot += 1;
    }

    let limit = ctx
        .stage_restrictions
        .get(&stage_id)
        .ok_or(Fault::KeyNotFound {
            site: "deploy_limit_reached",
            key: stage_id as i64,
        })?
        .deploy_limit;

    Ok(limit != 0 && deployed >= limit)
}
