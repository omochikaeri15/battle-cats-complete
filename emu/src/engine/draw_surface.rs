use super::{DrawSink, Surface};

pub fn draw_surface(dc: &mut dyn DrawSink, surface: Surface<'_>, x: i32, y: i32) {
    dc.draw_surface(surface, x, y);
}
