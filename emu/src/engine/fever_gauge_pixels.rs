use crate::{Fault, operation};

use super::{AppContext, get_global_map_id, get_special_rule_params};

pub fn fever_gauge_pixels(ctx: &mut AppContext, score: i32) -> Result<i32, Fault> {
    if ctx.special_rules.gauge_fill != -1 {
        return Ok(ctx.special_rules.gauge_fill);
    }

    let map_id = get_global_map_id(ctx, 0)?;
    let Some(params) = get_special_rule_params(ctx, &ctx.special_rules, map_id, 0xc)? else {
        return Ok(0);
    };
    let span = *params.get(1).ok_or(Fault::out_of_range())?;
    let fill = operation::cvttsd2si(
        (score as f64 - ctx.special_rules.point_baseline as f64) * 294.0 / span as f64,
    );

    ctx.special_rules.gauge_fill = fill;

    Ok(fill)
}
