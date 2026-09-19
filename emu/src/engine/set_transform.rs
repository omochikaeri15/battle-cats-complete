use super::DrawSink;

pub fn set_transform(dc: &mut dyn DrawSink, angle: f32, matrix: &[f32; 6]) {
    dc.set_transform(angle, matrix);
}
