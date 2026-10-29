use crate::{Fault, ops};

use super::{AppContext, is_tablet};

pub fn get_bottom_inset_logical(ctx: &mut AppContext) -> Result<i32, Fault> {
    let mut inset = 0i32;

    let notched = ctx.screen_metrics.inset_left != 0
        || ctx.screen_metrics.inset_top != 0
        || ctx.screen_metrics.inset_right != 0
        || ctx.screen_metrics.inset_bottom != 0;

    if notched && is_tablet(ctx)? != 0 {
        let design_h2 = ctx.screen_metrics.design_h2;

        if design_h2 <= ctx.screen_metrics.design_h {
            let scaled = (ctx.screen_metrics.inset_bottom as f32) * (design_h2 as f32)
                / (ctx.screen_metrics.screen_h as f32);

            inset = ops::cvttss2si(scaled.ceil());
        }
    }

    Ok(inset)
}
