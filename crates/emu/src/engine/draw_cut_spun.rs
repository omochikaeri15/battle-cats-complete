use super::{DrawSink, Imgcut};

pub fn draw_cut_spun(
    dc: &mut dyn DrawSink,
    sheet: &Imgcut,
    x: i32,
    y: i32,
    align: i32,
    pivot_x: i32,
    angle: f32,
    pivot_y: i32,
    pivot_align: i32,
    cut: i32,
) {
    dc.draw_cut_spun(
        sheet,
        x,
        y,
        angle,
        align,
        pivot_x,
        pivot_y,
        pivot_align,
        cut,
    );
}
