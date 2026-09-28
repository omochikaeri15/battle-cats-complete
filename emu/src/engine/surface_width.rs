use super::AppContext;

pub fn surface_width(ctx: &AppContext) -> i32 {
    ctx.screen_metrics.screen_w2
}
