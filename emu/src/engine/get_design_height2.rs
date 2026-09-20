use super::AppContext;

#[derive(Clone, Default, PartialEq, Debug)]
pub struct ScreenMetrics {
    pub screen_w: i32,
    pub screen_h: i32,
    pub screen_w2: i32,
    pub screen_h2: i32,
    pub design_w: i32,
    pub design_h2: i32,
    pub design_h: i32,
    pub inset_left: i32,
    pub inset_top: i32,
    pub inset_right: i32,
    pub inset_bottom: i32,
    pub letterbox_top: i32,
    pub scale2: f32,
    pub window_ratio: f32,
}

pub fn get_design_height2(ctx: &AppContext) -> i32 {
    ctx.screen_metrics.design_h2
}
