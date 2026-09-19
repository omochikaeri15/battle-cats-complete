use super::AppContext;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ScreenMetrics {
    pub design_height2: i32,
}

pub fn get_design_height2(ctx: &AppContext) -> i32 {
    ctx.screen_metrics.design_height2
}
