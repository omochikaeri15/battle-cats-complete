use crate::{operation, Fault};

use super::{
    draw_context, fill_polygon, fill_rect, get_design_height2, get_drawable_width, get_left_inset_logical, scene_ignores_insets, set_draw_scale, set_insets_ignored,
    set_tint, set_tint_alpha, AppContext,
};

fn wipe_inset(ctx: &AppContext) -> Result<i32, Fault> {
    if scene_ignores_insets(ctx)? != 0 {
        return Ok(0);
    }

    Ok(0i32.wrapping_sub(get_left_inset_logical(ctx)))
}

fn wipe_origin(ctx: &AppContext) -> Result<i32, Fault> {
    let width = get_drawable_width(ctx)?;
    let frame = ctx.i32_at(AppContext::FADE_FRAME)?;
    let swept = operation::div_neg_10(get_drawable_width(ctx)?.wrapping_mul(frame));

    Ok(swept.wrapping_add(width).wrapping_add(wipe_inset(ctx)?))
}

fn unwipe_origin(ctx: &AppContext) -> Result<i32, Fault> {
    let width = get_drawable_width(ctx)?;
    let frame = ctx.i32_at(AppContext::FADE_FRAME)?.wrapping_add(-1);
    let swept = operation::div_neg_10(get_drawable_width(ctx)?.wrapping_mul(frame));

    Ok(swept.wrapping_add(width))
}

fn spike(ctx: &mut AppContext, x: [i32; 3], y: [i32; 3]) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::POLYGON_XS, x[0])?;
    ctx.set_i32_at(AppContext::POLYGON_YS, y[0])?;
    ctx.set_i32_at(AppContext::POLYGON_XS + 4, x[1])?;
    ctx.set_i32_at(AppContext::POLYGON_YS + 4, y[1])?;
    ctx.set_i32_at(AppContext::POLYGON_XS + 8, x[2])?;
    ctx.set_i32_at(AppContext::POLYGON_YS + 8, y[2])?;

    let xs = [
        ctx.i32_at(AppContext::POLYGON_XS)?,
        ctx.i32_at(AppContext::POLYGON_XS + 4)?,
        ctx.i32_at(AppContext::POLYGON_XS + 8)?,
    ];
    let ys = [
        ctx.i32_at(AppContext::POLYGON_YS)?,
        ctx.i32_at(AppContext::POLYGON_YS + 4)?,
        ctx.i32_at(AppContext::POLYGON_YS + 8)?,
    ];

    fill_polygon(draw_context(&mut ctx.draw)?, &xs, &ys, 3);

    Ok(())
}

fn curtain_fill(ctx: &mut AppContext, x: i32, width: i32) -> Result<(), Fault> {
    let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
    let height = get_design_height2(ctx);

    fill_rect(draw_context(&mut ctx.draw)?, x, top, width, height);

    Ok(())
}

pub fn draw_screen_transition(ctx: &mut AppContext, closing: i32) -> Result<(), Fault> {
    if ctx.u8_at(AppContext::CURTAIN_ACTIVE)? == 0 {
        return Ok(());
    }

    let scale = ctx.screen_metrics.scale2;

    set_draw_scale(draw_context(&mut ctx.draw)?, scale);

    if closing == 0 {
        set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);

        let frame = ctx.i32_at(AppContext::FADE_FRAME)?;
        let alpha = if frame <= 0xb {
            frame.wrapping_mul(0x15)
        } else if frame == 0xc || frame as u32 > 0x18 {
            if frame == 0xc {
                0xff
            } else {
                0
            }
        } else {
            frame.wrapping_mul(-0x15).wrapping_add(0x210)
        };

        set_tint_alpha(draw_context(&mut ctx.draw)?, alpha);
        set_insets_ignored(ctx, 1)?;

        let width = get_drawable_width(ctx)?;

        curtain_fill(ctx, 0, width)?;
        set_insets_ignored(ctx, 0)?;

        return Ok(());
    }

    if closing != 1 {
        return Ok(());
    }

    set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);

    let frame = ctx.i32_at(AppContext::FADE_FRAME)?;

    if frame <= 0xa {
        let mut row: i32 = -0x180;

        while row != 0x4ac {
            let base = wipe_origin(ctx)?;

            spike(
                ctx,
                [base.wrapping_add(-0x6e), base, base],
                [row.wrapping_add(0x37), row, row.wrapping_add(0x6e)],
            )?;

            row += 0x9e;
        }

        let mut row: i32 = -0x112;

        while row != 0x51a {
            let base = wipe_origin(ctx)?;

            spike(
                ctx,
                [base.wrapping_add(-0x37), base, base],
                [row.wrapping_add(0x18), row, row.wrapping_add(0x30)],
            )?;

            row += 0x9e;
        }

        set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);

        let x = wipe_origin(ctx)?;
        let width = get_drawable_width(ctx)?.wrapping_mul(2);

        curtain_fill(ctx, x, width)?;

        return Ok(());
    }

    if (frame as u32) < 0xd {
        set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);
        set_insets_ignored(ctx, 1)?;

        let width = get_drawable_width(ctx)?;

        curtain_fill(ctx, 0, width)?;
        set_insets_ignored(ctx, 0)?;

        return Ok(());
    }

    let mut row: i32 = -0x180;

    while row != 0x4ac {
        let width = get_drawable_width(ctx)?;
        let base = unwipe_origin(ctx)?.wrapping_add(width);

        spike(
            ctx,
            [base.wrapping_add(0xdc), base.wrapping_add(0x6e), base.wrapping_add(0x6e)],
            [row.wrapping_add(0x37), row, row.wrapping_add(0x6e)],
        )?;

        row += 0x9e;
    }

    let mut row: i32 = -0x112;

    while row != 0x51a {
        let width = get_drawable_width(ctx)?;
        let base = unwipe_origin(ctx)?.wrapping_add(width);

        spike(
            ctx,
            [base.wrapping_add(0xa5), base.wrapping_add(0x6e), base.wrapping_add(0x6e)],
            [row.wrapping_add(0x18), row, row.wrapping_add(0x30)],
        )?;

        row += 0x9e;
    }

    set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);

    let x = unwipe_origin(ctx)?;
    let width = get_drawable_width(ctx)?.wrapping_add(0x6e);

    curtain_fill(ctx, x, width)?;

    Ok(())
}
