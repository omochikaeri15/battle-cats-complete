use super::{DrawSink, Imgcut};

pub fn draw_cut(dc: &mut dyn DrawSink, sheet: &Imgcut, x: i32, y: i32, cut: i32) {
    dc.draw_cut(sheet, x, y, cut);
}
