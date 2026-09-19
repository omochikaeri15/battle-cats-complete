use super::{DrawSink, Imgcut};

#[allow(clippy::too_many_arguments)]
pub fn draw_panel(
    dc: &mut dyn DrawSink,
    sheet: &Imgcut,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    scale: f32,
    cut_a: i32,
    cut_b: i32,
) {
    dc.draw_panel(sheet, x, y, width, height, scale, cut_a, cut_b);
}
