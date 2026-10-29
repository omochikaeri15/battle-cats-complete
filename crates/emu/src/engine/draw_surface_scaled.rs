use super::{DrawSink, Surface};

pub fn draw_surface_scaled(
    dc: &mut dyn DrawSink,
    surface: Surface<'_>,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) {
    dc.draw_surface_scaled(surface, x, y, width, height);
}
