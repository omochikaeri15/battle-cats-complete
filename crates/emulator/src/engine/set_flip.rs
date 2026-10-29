use super::DrawSink;

pub fn set_flip(dc: &mut dyn DrawSink, flip: i32) {
    dc.set_flip(flip);
}
