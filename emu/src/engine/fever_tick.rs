use crate::{Fault, operation};

use super::{
    AppContext, get_global_map_id, get_special_rule_params, play_sound, refresh_cat_speeds,
    sound_manager,
};

const SITE: &str = "fever_tick";

pub fn fever_tick(ctx: &mut AppContext, points: i32) -> Result<(), Fault> {
    let map_id = get_global_map_id(ctx, 0)?;
    let Some(params) = get_special_rule_params(ctx, &ctx.special_rules, map_id, 0xc)?.cloned()
    else {
        return Ok(());
    };

    let remaining = ctx.special_rules.fever_count;

    if remaining > 0 {
        ctx.special_rules.fever_count = remaining.wrapping_sub(1);

        if remaining.wrapping_sub(1) != 0 {
            return Ok(());
        }

        ctx.special_rules.point_baseline = points;

        return refresh_cat_speeds(ctx);
    }

    let fever_rule = *params.first().ok_or(Fault::IndexOutOfRange {
        site: SITE,
        index: 0,
        limit: params.len() as i64,
    })?;

    if fever_rule != 0 {
        return Ok(());
    }

    let needed = *params.get(1).ok_or(Fault::IndexOutOfRange {
        site: SITE,
        index: 1,
        limit: params.len() as i64,
    })?;
    let baseline = ctx.special_rules.point_baseline;
    let target = operation::cvttsd2si((points as f64 - baseline as f64) * 294.0 / needed as f64);
    let fill = ctx.special_rules.gauge_fill;
    let mid = operation::div_2(fill.wrapping_add(target));
    let step = if mid.wrapping_sub(fill) >= 2 {
        mid
    } else {
        fill.wrapping_add(1)
    };

    ctx.special_rules.gauge_fill = if fill >= target { target } else { step };

    if baseline.wrapping_add(needed) > points {
        return Ok(());
    }

    let length = *params.get(2).ok_or(Fault::IndexOutOfRange {
        site: SITE,
        index: 2,
        limit: params.len() as i64,
    })?;

    ctx.special_rules.fever_count = length;
    ctx.special_rules.fever_frame = 0;
    ctx.special_rules.fade_phase = 0;
    ctx.special_rules.fade_reverse = 0;
    ctx.special_rules.gauge_fill = -1;

    refresh_cat_speeds(ctx)?;
    play_sound(sound_manager(ctx)?, 0xc2, None);

    Ok(())
}
