use crate::Fault;

use super::{
    AppContext, get_global_map_id, get_map_type, get_special_rule, is_score_stage, powerups_cleared,
};

pub fn powerup_disabled(ctx: &mut AppContext, powerup: i32) -> Result<bool, Fault> {
    if powerup == 2 {
        let map_id = get_global_map_id(ctx, 0)?;

        if get_special_rule(ctx, &ctx.special_rules, map_id, 0)? {
            return Ok(true);
        }
    }

    if get_map_type(ctx, 0)? == -0x18 {
        return Ok(powerup != 0);
    }

    if powerups_cleared(ctx)? && (powerup as u32 > 4 || 0x1a >> powerup & 1 == 0) {
        return Ok(true);
    }

    if !is_score_stage(ctx.event_items.as_ref()) {
        return Ok(false);
    }

    Ok(powerup != 0 && powerup != 3)
}
