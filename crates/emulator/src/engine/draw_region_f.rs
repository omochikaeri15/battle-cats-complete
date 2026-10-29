use super::{DrawSink, Imgcut};

pub fn draw_region_f(
    dc: &mut dyn DrawSink,
    sheet: &Imgcut,
    src_x: i32,
    src_y: i32,
    src_w: i32,
    src_h: i32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    dc.draw_region_f(sheet, src_x, src_y, src_w, src_h, x, y, width, height);
}
