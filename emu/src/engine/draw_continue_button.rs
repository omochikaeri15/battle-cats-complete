use crate::{Fault, operation};

use super::{AppContext, DECK_PRESS_SIZE_TABLE, Imgcut, draw_context, draw_cut_scaled};

const SITE: &str = "draw_continue_button";

pub fn draw_continue_button(
    ctx: &mut AppContext,
    sheet: &Imgcut,
    x: i32,
    mode: i32,
    anchored: i32,
) -> Result<(), Fault> {
    let step = ctx.i32_at(AppContext::LOSE_SHOP_PRESS)?;
    let bounce = DECK_PRESS_SIZE_TABLE
        .get(step as i64 as usize)
        .copied()
        .ok_or(Fault::IndexOutOfRange {
            site: SITE,
            index: step as i64,
            limit: DECK_PRESS_SIZE_TABLE.len() as i64,
        })?;
    let origin = if anchored == 0 {
        0
    } else {
        ctx.i32_at(AppContext::LOSE_SHOP_X)?
    };
    let half = operation::div_2(bounce);
    let across = origin.wrapping_add(x.wrapping_sub(half));
    let down = 0x24ci32.wrapping_sub(half);

    draw_cut_scaled(
        draw_context(&mut ctx.draw)?,
        sheet,
        across.wrapping_add(0x12),
        down,
        bounce.wrapping_add(0x37),
        bounce.wrapping_add(0x2a),
        0x15,
    );

    if mode == 2 || (mode == 1 && ctx.i32_at(AppContext::CAT_FOOD_SHOP_ENABLED)? > 0) {
        draw_cut_scaled(
            draw_context(&mut ctx.draw)?,
            sheet,
            across.wrapping_add(0x3c),
            down,
            bounce.wrapping_add(0x1b),
            bounce.wrapping_add(0x1a),
            0x12,
        );
    }

    let down = 0x269i32.wrapping_sub(half);

    draw_cut_scaled(
        draw_context(&mut ctx.draw)?,
        sheet,
        across.wrapping_add(0xa),
        down,
        bounce.wrapping_add(0x4c),
        bounce.wrapping_add(0x19),
        0x18,
    );

    Ok(())
}
