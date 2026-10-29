use super::DrawSink;

pub fn set_color(dc: &mut dyn DrawSink, red: i32, green: i32, blue: i32, alpha: i32) {
    dc.set_color(red, green, blue, alpha);
}
