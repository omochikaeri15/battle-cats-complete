use crate::Fault;

use super::{DrawSink, Imgcut, NumberBox, draw_number};

pub fn draw_number_scaled(
    dc: &mut dyn DrawSink,
    sheet: &Imgcut,
    base: i32,
    value: i32,
    offset: i32,
    x: f32,
    y: f32,
    spacing: f32,
    scale: f32,
    lead: i32,
    flags: i32,
    digits: i32,
) -> Result<NumberBox, Fault> {
    draw_number(
        dc, sheet, base, value, offset, x, y, spacing, scale, scale, lead, flags, digits,
    )
}
