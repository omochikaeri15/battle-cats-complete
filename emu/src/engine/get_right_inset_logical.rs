use crate::{Fault, operation};

use super::AppContext;

pub fn get_right_inset_logical(ctx: &AppContext) -> Result<i32, Fault> {
    let metrics = &ctx.screen_metrics;

    if metrics.inset_left == 0
        && metrics.inset_top == 0
        && metrics.inset_right == 0
        && metrics.inset_bottom == 0
    {
        return Ok(0);
    }

    let width = metrics.design_w;
    let drawable = if metrics.inset_left != 0 || metrics.inset_right != 0 {
        let dividend = metrics
            .screen_w
            .wrapping_sub(metrics.inset_left.wrapping_add(metrics.inset_right))
            .wrapping_mul(metrics.design_h);
        let divisor = metrics
            .screen_h
            .wrapping_sub(metrics.inset_top.wrapping_add(metrics.inset_bottom));

        operation::idiv(dividend, divisor).ok_or(Fault::divide(divisor as i64))?
    } else {
        metrics.design_w
    };

    let left = operation::cvttss2si(
        ((width as f32) * (metrics.inset_left as f32) / (metrics.screen_w as f32)).ceil(),
    );

    Ok(width.wrapping_sub(drawable.wrapping_add(left)))
}
