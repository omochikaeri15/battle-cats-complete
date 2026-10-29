use super::DrawSink;

pub fn fill_polygon(dc: &mut dyn DrawSink, xs: &[i32], ys: &[i32], count: i32) {
    dc.fill_polygon(xs, ys, count);
}
