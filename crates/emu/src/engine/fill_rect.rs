use super::DrawSink;

pub fn fill_rect(dc: &mut dyn DrawSink, x: i32, y: i32, width: i32, height: i32) {
    dc.fill_rect(x, y, width, height);
}
