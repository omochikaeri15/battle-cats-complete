use crate::operation;

use super::AppContext;

pub fn get_top_inset_offset(ctx: &AppContext) -> i32 {
    let metrics = &ctx.screen_metrics;

    if metrics.inset_left == 0
        && metrics.inset_top == 0
        && metrics.inset_right == 0
        && metrics.inset_bottom == 0
    {
        return 0;
    }

    let scaled = operation::cvttss2si(
        ((metrics.design_h2 as f32) * (metrics.inset_top as f32) / (metrics.screen_h as f32))
            .ceil(),
    );

    metrics
        .design_h2
        .wrapping_sub(scaled.wrapping_add(metrics.letterbox_top))
}
