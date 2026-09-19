use super::AppContext;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ScreenMetrics {
    pub screen_w: i32,
    pub screen_h: i32,
    pub design_w: i32,
    pub design_h2: i32,
    pub design_h: i32,
    pub inset_left: i32,
    pub inset_top: i32,
    pub inset_right: i32,
    pub inset_bottom: i32,
}

pub fn get_design_height2(ctx: &AppContext) -> i32 {
    ctx.screen_metrics.design_h2
}
