use super::DrawSink;

pub fn dc_begin_frame(dc: &mut dyn DrawSink) {
    dc.begin_frame();
}
