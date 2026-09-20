use crate::{Fault, operation};

use super::{AppContext, scene_ignores_insets};

pub fn get_drawable_width(ctx: &AppContext) -> Result<i32, Fault> {
    let metrics = &ctx.screen_metrics;

    if metrics.inset_left == 0
        && metrics.inset_top == 0
        && metrics.inset_right == 0
        && metrics.inset_bottom == 0
    {
        return Ok(metrics.design_w);
    }

    if scene_ignores_insets(ctx)? != 0 {
        return Ok(metrics.design_w);
    }

    let left = metrics.inset_left;
    let right = metrics.inset_right;

    if left | right == 0 {
        return Ok(metrics.design_w);
    }

    let dividend = metrics
        .screen_w
        .wrapping_sub(right.wrapping_add(left))
        .wrapping_mul(metrics.design_h);
    let divisor = metrics
        .screen_h
        .wrapping_sub(metrics.inset_top.wrapping_add(metrics.inset_bottom));

    operation::idiv(dividend, divisor).ok_or(Fault::divide(divisor as i64))
}
