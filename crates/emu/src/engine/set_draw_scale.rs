use super::DrawSink;

pub fn set_draw_scale(dc: &mut dyn DrawSink, scale: f32) {
    dc.set_draw_scale(scale);
}
