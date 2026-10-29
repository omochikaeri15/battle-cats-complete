use super::DrawSink;

pub fn fill_rect_f(dc: &mut dyn DrawSink, x: f32, y: f32, width: f32, height: f32) {
    dc.fill_rect_f(x, y, width, height);
}
