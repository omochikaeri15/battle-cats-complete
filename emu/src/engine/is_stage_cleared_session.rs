use crate::Fault;

use super::{get_stage_record, is_conditioned_map, map_index_of_map_id, map_type_of_map_id, stage_condition_met, AppContext};

pub fn is_stage_cleared_session(ctx: &mut AppContext, map: i32, stage: i32, variant: i32) -> Result<bool, Fault> {
    if map_type_of_map_id(map) == -11 && stage_condition_met(ctx, -11, map_index_of_map_id(map), stage)? {
        return Ok(true);
    }

    if variant & !1 == 2 {
        let mut key = variant.wrapping_add(map.wrapping_mul(0x3e8));
        let mut step = -1i32;

        loop {
            if ctx.cleared_session_keys.contains(&key) {
                return Ok(true);
            }

            step = step.wrapping_add(1);
            key = key.wrapping_add(0xa);

            if step == 0x30 {
                return Ok(false);
            }
        }
    }

    let key = map.wrapping_mul(0x3e8).wrapping_add(stage.wrapping_mul(5).wrapping_mul(2)).wrapping_add(variant);
    let found = ctx.cleared_session_keys.contains(&key);

    if variant == 0 && !found && get_stage_record(ctx, map_type_of_map_id(map), map_index_of_map_id(map), stage, 0, 0)? > 0 {
        ctx.cleared_session_keys.push(key);

        return Ok(true);
    }

    if is_conditioned_map(ctx, map) && stage_condition_met(ctx, map_type_of_map_id(map), map_index_of_map_id(map), stage)? {
        return Ok(true);
    }

    Ok(found)
}
