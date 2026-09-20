use super::{DrawSink, Imgcut};

pub fn draw_nine_slice(
    dc: &mut dyn DrawSink,
    sheet: &Imgcut,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    scale: f32,
    cut: i32,
    border_x: i32,
    border_y: i32,
    inner_w: i32,
    inner_h: i32,
) {
    dc.draw_nine_slice(
        sheet, x, y, width, height, scale, cut, border_x, border_y, inner_w, inner_h,
    );
}
