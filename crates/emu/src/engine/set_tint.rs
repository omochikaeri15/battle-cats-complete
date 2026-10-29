use super::DrawSink;

pub fn set_tint(dc: &mut dyn DrawSink, red: i32, green: i32, blue: i32, alpha: i32) {
    dc.set_tint(red, green, blue, alpha);
}
