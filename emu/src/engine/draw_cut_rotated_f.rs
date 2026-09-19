use super::{DrawSink, Imgcut};

#[allow(clippy::too_many_arguments)]
pub fn draw_cut_rotated_f(
    dc: &mut dyn DrawSink,
    sheet: &Imgcut,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    pivot_x: f32,
    pivot_y: f32,
    angle: f32,
    align: i32,
    pivot_align: i32,
    cut: i32,
) {
    dc.draw_cut_rotated_f(
        sheet,
        x,
        y,
        width,
        height,
        pivot_x,
        pivot_y,
        angle,
        align,
        pivot_align,
        cut,
    );
}
