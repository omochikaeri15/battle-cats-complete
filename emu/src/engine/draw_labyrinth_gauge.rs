use crate::{Fault, operation};

use super::{
    AppContext, Imgcut, digit_count, draw_context, draw_cut, draw_number_plain, draw_region,
    get_labyrinth_map_id, get_stage_count, imgcut_get_sprite_cut, map_index_of_map_id,
};

const SITE: &str = "draw_labyrinth_gauge";

pub fn draw_labyrinth_gauge(
    ctx: &mut AppContext,
    gauge: &Imgcut,
    digits: &Imgcut,
    x: i32,
    y: i32,
    floor: i32,
) -> Result<(), Fault> {
    draw_cut(draw_context(&mut ctx.draw)?, gauge, x, y, 0);

    let map = map_index_of_map_id(get_labyrinth_map_id(ctx)?);
    let total = get_stage_count(ctx, -0x15, map)?.wrapping_add(-1);
    let across = operation::idiv(floor.wrapping_mul(0x75), total)
        .ok_or(Fault::divide(SITE, total as i64))?
        .wrapping_add(x);
    let raised = floor.wrapping_mul(0xd0);
    let down = operation::div_100(raised).wrapping_add(y);
    let cut = *imgcut_get_sprite_cut(gauge, 9)?;
    let src_x = cut[0];
    let src_y = cut[1];
    let src_w = operation::div_100(floor.wrapping_mul(-0x75)).wrapping_add(cut[2]);
    let src_h = cut[3];

    draw_region(
        draw_context(&mut ctx.draw)?,
        gauge,
        across,
        down,
        src_x,
        src_y,
        src_w,
        src_h,
    );

    let edge = imgcut_get_sprite_cut(gauge, 9)?[2].wrapping_add(x);
    let map = map_index_of_map_id(get_labyrinth_map_id(ctx)?);
    let total = get_stage_count(ctx, -0x15, map)?.wrapping_add(-1);
    let lift = operation::idiv(raised, total).ok_or(Fault::divide(SITE, total as i64))?;
    let places = digit_count(floor);

    draw_cut(
        draw_context(&mut ctx.draw)?,
        gauge,
        edge.wrapping_add(-1),
        lift.wrapping_add(y).wrapping_add(-0x1c),
        1,
    );

    let nudge = i32::from(places >= 2).wrapping_mul(4);
    let label = edge.wrapping_add(nudge).wrapping_add(0x1d) as f32;
    let baseline = lift.wrapping_add(y).wrapping_add(-0xb) as f32;

    draw_number_plain(
        draw_context(&mut ctx.draw)?,
        digits,
        0x6e,
        floor,
        0,
        label,
        baseline,
        -2.0,
        0,
        2,
        0,
    )?;

    Ok(())
}
