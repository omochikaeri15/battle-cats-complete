use crate::{Fault, ops};

use super::AppContext;

pub fn window_to_design(ctx: &AppContext, x: &mut i32, y: &mut i32) -> Result<(), Fault> {
    let metrics = &ctx.screen_metrics;

    *x = ops::idiv(x.wrapping_mul(metrics.design_w), metrics.screen_w).ok_or(Fault::divide(metrics.screen_w as i64))?;
    *y = ops::idiv(y.wrapping_mul(metrics.design_h2), metrics.screen_h).ok_or(Fault::divide(metrics.screen_h as i64))?;

    Ok(())
}
