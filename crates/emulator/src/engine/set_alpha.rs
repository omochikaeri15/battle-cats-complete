use super::DrawSink;

pub fn set_alpha(dc: &mut dyn DrawSink, alpha: i32) {
    dc.set_alpha(alpha);
}
