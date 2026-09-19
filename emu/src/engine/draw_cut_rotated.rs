use super::{DrawSink, Imgcut};

#[allow(clippy::too_many_arguments)]
pub fn draw_cut_rotated(dc: &mut dyn DrawSink, sheet: &Imgcut, x: i32, y: i32, width: i32, height: i32, angle: f32, align: i32, pivot_x: i32, pivot_y: i32, pivot_align: i32, cut: i32) {
    dc.draw_cut_rotated(sheet, x, y, width, height, angle, align, pivot_x, pivot_y, pivot_align, cut);
}
