use crate::{operation, Fault};

use super::{digit_count, draw_context, draw_cut, draw_number_plain, draw_region, imgcut_get_sprite_cut, AppContext, Imgcut};

pub fn draw_percent_number(ctx: &mut AppContext, sheet: &Imgcut, digits: &Imgcut, x: i32, y: i32, percent: i32) -> Result<(), Fault> {
    draw_cut(draw_context(&mut ctx.draw)?, sheet, x, y, 0);

    let across = operation::div_100(percent.wrapping_mul(-0x71)).wrapping_add(x).wrapping_add(0x9c);
    let rise = operation::div_100(percent.wrapping_mul(0xd8));
    let down = rise.wrapping_add(y);
    let gauge = *imgcut_get_sprite_cut(sheet, 9)?;
    let cut_x = gauge[0];
    let cut_y = gauge[1];
    let cut_w = operation::div_100(0x64i32.wrapping_sub(percent).wrapping_mul(-0x71)).wrapping_add(gauge[2]);
    let cut_h = gauge[3];

    draw_region(draw_context(&mut ctx.draw)?, sheet, across, down, cut_x, cut_y, cut_w, cut_h);

    let width = imgcut_get_sprite_cut(sheet, 9)?[2];
    let edge = x.wrapping_add(width);

    draw_cut(draw_context(&mut ctx.draw)?, sheet, edge.wrapping_add(0x2a), y.wrapping_add(rise).wrapping_add(-0x1c), 1);

    let places = digit_count(percent);
    let half = operation::div_2(imgcut_get_sprite_cut(sheet, 1)?[2]);
    let start = half.wrapping_add(edge).wrapping_add(0x2a);
    let span = places.wrapping_mul(11);
    let mark = imgcut_get_sprite_cut(sheet, 0x10)?[2];
    let centre = operation::div_2(span.wrapping_add(mark).wrapping_add(1))
        .wrapping_add(start)
        .wrapping_sub(imgcut_get_sprite_cut(sheet, 0x10)?[2]);

    draw_cut(draw_context(&mut ctx.draw)?, sheet, centre, y.wrapping_add(rise).wrapping_add(-1), 0x10);

    let tail = centre.wrapping_add(-1);

    draw_number_plain(
        draw_context(&mut ctx.draw)?,
        digits,
        0x6e,
        percent,
        0,
        tail as f32,
        y.wrapping_add(rise).wrapping_add(-0xb) as f32,
        -2.0,
        0,
        2,
        0,
    )?;

    Ok(())
}
