use crate::{Fault, operation};

use super::{DrawSink, Imgcut, digit_count, draw_region_f, imgcut_get_sprite_cut};

#[derive(Clone, Copy, Default, Debug)]
pub struct NumberBox {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub digits: i32,
}

pub fn draw_number(
    dc: &mut dyn DrawSink,
    sheet: &Imgcut,
    base: i32,
    value: i32,
    offset: i32,
    x: f32,
    y: f32,
    spacing: f32,
    scale_x: f32,
    scale_y: f32,
    lead: i32,
    flags: i32,
    digits: i32,
) -> Result<NumberBox, Fault> {
    let digits = if flags & 0x10 == 0 {
        digit_count(value)
    } else {
        digits
    };
    let start = lead.wrapping_add(offset) as f32;
    let mut width = digits.wrapping_sub(1) as f32 * spacing + start;

    if digits > 0 {
        let mut place = digits.wrapping_sub(1);

        loop {
            let power = operation::cvttss2si(operation::powf(10.0, place as f32));
            let shifted = operation::idiv(value, power).ok_or(Fault::divide(power as i64))?;
            let digit = shifted
                .wrapping_sub(operation::div_10(shifted).wrapping_mul(10))
                .wrapping_add(base);

            width += imgcut_get_sprite_cut(sheet, digit)?[2] as f32 * scale_x;

            if place == 0 {
                break;
            }

            place = place.wrapping_sub(1);
        }
    }

    let mut left = offset as f32 + x;

    if flags & 1 != 0 {
        left += width * -0.5;
    } else {
        left -= if flags & 2 != 0 { width } else { 0.0 };
    }

    let mut top = y;

    if flags & 4 != 0 {
        top += imgcut_get_sprite_cut(sheet, base)?[3] as f32 * scale_y * -0.5;
    } else if flags & 8 != 0 {
        top -= imgcut_get_sprite_cut(sheet, base)?[3] as f32 * scale_y;
    }

    let height = imgcut_get_sprite_cut(sheet, base)?[3];
    let area = NumberBox {
        left,
        right: width + left - start,
        top,
        bottom: height as f32 * scale_y + top,
        digits,
    };

    if digits <= 0 {
        return Ok(area);
    }

    let mut place = digits.wrapping_sub(1);
    let mut cursor = left;

    loop {
        let power = operation::cvttss2si(operation::powf(10.0, place as f32));
        let shifted = operation::idiv(value, power).ok_or(Fault::divide(power as i64))?;
        let digit = shifted
            .wrapping_sub(operation::div_10(shifted).wrapping_mul(10))
            .wrapping_add(base);
        let cut_width = imgcut_get_sprite_cut(sheet, digit)?[2] as f32 * scale_x;
        let cut_height = imgcut_get_sprite_cut(sheet, digit)?[3] as f32 * scale_y;
        let cut = *imgcut_get_sprite_cut(sheet, digit)?;

        draw_region_f(
            dc, sheet, cut[0], cut[1], cut[2], cut[3], cursor, top, cut_width, cut_height,
        );

        cursor += imgcut_get_sprite_cut(sheet, digit)?[2] as f32 * scale_x + spacing;

        if place == 0 {
            break;
        }

        place = place.wrapping_sub(1);
    }

    Ok(area)
}
