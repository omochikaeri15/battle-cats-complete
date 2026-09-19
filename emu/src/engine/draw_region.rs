use super::{DrawSink, Imgcut};

#[allow(clippy::too_many_arguments)]
pub fn draw_region(
    dc: &mut dyn DrawSink,
    sheet: &Imgcut,
    x: i32,
    y: i32,
    src_x: i32,
    src_y: i32,
    src_w: i32,
    src_h: i32,
) {
    dc.draw_region(sheet, x, y, src_x, src_y, src_w, src_h);
}
