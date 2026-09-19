use crate::Fault;

use super::{AppContext, get_global_map_id, get_scene_id, get_special_rule_params};

pub fn fever_fade_tick(ctx: &mut AppContext) -> Result<(), Fault> {
    let map_id = get_global_map_id(ctx, 0)?;
    let Some(params) = get_special_rule_params(ctx, &ctx.special_rules, map_id, 0xc)?.cloned()
    else {
        return Ok(());
    };

    if get_scene_id(ctx)? != 0x12c {
        return Ok(());
    }

    let remaining = ctx.special_rules.fever_count;

    if remaining <= 0 {
        return Ok(());
    }

    ctx.special_rules.fever_frame = ctx.special_rules.fever_frame.wrapping_add(1);

    if ctx.special_rules.fade_reverse != 0 {
        let phase = ctx.special_rules.fade_phase;

        if phase <= 4 {
            ctx.special_rules.fade_phase = phase.wrapping_add(1);
        }

        return Ok(());
    }

    let length = *params.get(2).ok_or(Fault::IndexOutOfRange {
        site: "fever_fade_tick",
        index: 2,
        limit: params.len() as i64,
    })?;

    ctx.special_rules.fade_phase = if length.wrapping_sub(remaining) >= length.wrapping_add(-5) {
        6i32.wrapping_sub(remaining)
    } else {
        0
    };

    Ok(())
}
