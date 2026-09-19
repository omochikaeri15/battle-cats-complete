use crate::operation;

use super::AppContext;

pub fn get_left_inset_logical(ctx: &AppContext) -> i32 {
    let metrics = &ctx.screen_metrics;

    if metrics.inset_left == 0 && metrics.inset_top == 0 && metrics.inset_right == 0 && metrics.inset_bottom == 0 {
        return 0;
    }

    operation::cvttss2si(((metrics.design_w as f32) * (metrics.inset_left as f32) / (metrics.screen_w as f32)).ceil())
}
