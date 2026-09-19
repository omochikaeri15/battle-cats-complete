use super::{DrawSink, Surface};

pub fn draw_surface_aligned(dc: &mut dyn DrawSink, surface: Surface<'_>, x: i32, y: i32, align: i32) {
    dc.draw_surface_aligned(surface, x, y, align);
}
