use super::AppContext;

pub fn has_insets(ctx: &AppContext) -> bool {
    let metrics = &ctx.screen_metrics;

    metrics.inset_left != 0
        || metrics.inset_top != 0
        || metrics.inset_right != 0
        || metrics.inset_bottom != 0
}
