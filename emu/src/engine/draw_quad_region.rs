use super::{DrawSink, Imgcut};

pub fn draw_quad_region(
    dc: &mut dyn DrawSink,
    sheet: &Imgcut,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    x3: i32,
    y3: i32,
    src_x: i32,
    src_y: i32,
    src_w: i32,
    src_h: i32,
) {
    dc.draw_quad_region(
        sheet, x0, y0, x1, y1, x2, y2, x3, y3, src_x, src_y, src_w, src_h,
    );
}
