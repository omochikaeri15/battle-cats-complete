use super::DrawSink;

pub fn dc_end_frame(dc: &mut dyn DrawSink) {
    dc.end_frame();
}
