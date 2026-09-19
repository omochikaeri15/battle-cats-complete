use super::DrawSink;

pub fn fill_polygon_colored(
    dc: &mut dyn DrawSink,
    xs: &[i32],
    ys: &[i32],
    colors: &[u32],
    count: i32,
) {
    dc.fill_polygon_colored(xs, ys, colors, count);
}
