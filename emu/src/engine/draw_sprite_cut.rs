use super::{DrawSink, Imgcut};

#[allow(clippy::too_many_arguments)]
pub fn draw_sprite_cut(
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
    cut: i32,
) {
    dc.draw_sprite_cut(sheet, x0, y0, x1, y1, x2, y2, x3, y3, cut);
}
