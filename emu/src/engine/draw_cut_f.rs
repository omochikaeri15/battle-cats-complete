use super::{DrawSink, Imgcut};

pub fn draw_cut_f(
    dc: &mut dyn DrawSink,
    sheet: &Imgcut,
    cut: i32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    dc.draw_cut_f(sheet, cut, x, y, width, height);
}
