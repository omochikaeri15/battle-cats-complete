use super::{DrawSink, Imgcut};

pub fn draw_cut_scaled(
    dc: &mut dyn DrawSink,
    sheet: &Imgcut,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    cut: i32,
) {
    dc.draw_cut_scaled(sheet, x, y, width, height, cut);
}
