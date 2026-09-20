use crate::{Fault, operation};

use super::{
    AppContext, draw_context, draw_cut, get_cannon_countdown, get_cannon_recharge,
    get_drawable_width, get_right_inset_logical, get_stage_record, get_top_inset_offset,
};

const GAUGE: [(i32, i32, i32); 9] = [
    (0xd, 0x76, 0xb),
    (0x1a, 0x6a, 0xc),
    (0x27, 0x5e, 0xd),
    (0x34, 0x52, 0xe),
    (0x41, 0x46, 0xf),
    (0x4e, 0x3a, 0x10),
    (0x5b, 0x2e, 0x11),
    (0x68, 0x22, 0x12),
    (0x75, 0x16, 0x13),
];

pub fn draw_cannon(ctx: &mut AppContext) -> Result<(), Fault> {
    if get_stage_record(ctx, -2, 0, 1, 0, 0)? <= 0 {
        return Ok(());
    }

    if get_cannon_countdown(ctx, 0)? == 0 {
        let x = ctx.i32_at(AppContext::CANNON_RECT)?;
        let y = ctx.i32_at(AppContext::CANNON_RECT + 4)?;

        if ctx.i32_at(AppContext::BLINK_ON)? != 0 {
            let y = y.wrapping_add(ctx.i32_at(AppContext::DECK_BAR_SLIDE)?);

            draw_cut(
                draw_context(&mut ctx.draw)?,
                ctx.img002_sheet
                    .as_deref()
                    .ok_or(Fault::null_pointer())?,
                x,
                y,
                7,
            );

            let x = get_drawable_width(ctx)?
                .wrapping_sub(get_right_inset_logical(ctx)?)
                .wrapping_add(-0x81);
            let row = ctx
                .i32_at(AppContext::DECK_BAR_SLIDE)?
                .wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
            let y = get_top_inset_offset(ctx)
                .wrapping_add(row)
                .wrapping_add(0x245);

            draw_cut(
                draw_context(&mut ctx.draw)?,
                ctx.img002_sheet
                    .as_deref()
                    .ok_or(Fault::null_pointer())?,
                x,
                y,
                0xa,
            );

            return Ok(());
        }

        let y = y
            .wrapping_add(ctx.i32_at(AppContext::DECK_BAR_SLIDE)?)
            .wrapping_add(0x76);

        draw_cut(
            draw_context(&mut ctx.draw)?,
            ctx.img002_sheet
                .as_deref()
                .ok_or(Fault::null_pointer())?,
            x,
            y,
            0xb,
        );

        for (_, offset, cut) in GAUGE.iter().skip(1) {
            let x = ctx.i32_at(AppContext::CANNON_RECT)?;
            let y = ctx
                .i32_at(AppContext::DECK_BAR_SLIDE)?
                .wrapping_add(ctx.i32_at(AppContext::CANNON_RECT + 4)?)
                .wrapping_add(*offset);

            draw_cut(
                draw_context(&mut ctx.draw)?,
                ctx.img002_sheet
                    .as_deref()
                    .ok_or(Fault::null_pointer())?,
                x,
                y,
                *cut,
            );
        }

        let x = ctx.i32_at(AppContext::CANNON_RECT)?;
        let y = ctx
            .i32_at(AppContext::DECK_BAR_SLIDE)?
            .wrapping_add(ctx.i32_at(AppContext::CANNON_RECT + 4)?);

        draw_cut(
            draw_context(&mut ctx.draw)?,
            ctx.img002_sheet
                .as_deref()
                .ok_or(Fault::null_pointer())?,
            x,
            y,
            0x14,
        );

        let x = get_drawable_width(ctx)?
            .wrapping_sub(get_right_inset_logical(ctx)?)
            .wrapping_add(-0x81);
        let row = ctx
            .i32_at(AppContext::DECK_BAR_SLIDE)?
            .wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let y = get_top_inset_offset(ctx)
            .wrapping_add(row)
            .wrapping_add(0x245);

        draw_cut(
            draw_context(&mut ctx.draw)?,
            ctx.img002_sheet
                .as_deref()
                .ok_or(Fault::null_pointer())?,
            x,
            y,
            9,
        );

        return Ok(());
    }

    let x = ctx.i32_at(AppContext::CANNON_RECT)?;
    let y = ctx
        .i32_at(AppContext::DECK_BAR_SLIDE)?
        .wrapping_add(ctx.i32_at(AppContext::CANNON_RECT + 4)?);

    draw_cut(
        draw_context(&mut ctx.draw)?,
        ctx.img002_sheet
            .as_deref()
            .ok_or(Fault::null_pointer())?,
        x,
        y,
        8,
    );

    let elapsed = get_cannon_recharge(ctx, 0)?.wrapping_sub(get_cannon_countdown(ctx, 0)?);
    let scaled = (elapsed << 7).wrapping_add(elapsed.wrapping_mul(2));
    let recharge = get_cannon_recharge(ctx, 0)?;

    ctx.set_i32_at(
        AppContext::DRAW_TEMP_0,
        operation::idiv(scaled, recharge).ok_or(Fault::divide(recharge as i64))?,
    )?;

    for (threshold, offset, cut) in GAUGE {
        if ctx.i32_at(AppContext::DRAW_TEMP_0)? < threshold {
            return Ok(());
        }

        let x = ctx.i32_at(AppContext::CANNON_RECT)?;
        let y = ctx
            .i32_at(AppContext::DECK_BAR_SLIDE)?
            .wrapping_add(ctx.i32_at(AppContext::CANNON_RECT + 4)?)
            .wrapping_add(offset);

        draw_cut(
            draw_context(&mut ctx.draw)?,
            ctx.img002_sheet
                .as_deref()
                .ok_or(Fault::null_pointer())?,
            x,
            y,
            cut,
        );
    }

    Ok(())
}
