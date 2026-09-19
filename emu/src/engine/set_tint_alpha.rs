use super::DrawSink;

pub fn set_tint_alpha(dc: &mut dyn DrawSink, alpha: i32) {
    dc.set_tint_alpha(alpha);
}
