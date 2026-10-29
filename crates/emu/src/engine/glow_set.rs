use super::DrawSink;

pub fn glow_set(dc: &mut dyn DrawSink, mode: i32) {
    dc.glow_set(mode);
}
