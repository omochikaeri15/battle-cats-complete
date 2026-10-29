use super::{DrawSink, Imgcut};

pub fn draw_region_scaled(
    dc: &mut dyn DrawSink,
    sheet: &Imgcut,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    src_x: i32,
    src_y: i32,
    src_w: i32,
    src_h: i32,
) {
    dc.draw_region_scaled(sheet, x, y, width, height, src_x, src_y, src_w, src_h);
}
