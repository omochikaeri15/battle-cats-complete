use super::DrawSink;

pub fn dc_set_viewport(dc: &mut dyn DrawSink, surface_w: i32, surface_h: i32, screen_w: i32, screen_h: i32, depth: f32) {
    dc.set_viewport(surface_w, surface_h, screen_w, screen_h, depth);
}
