use crate::{Fault, ops};

use super::{
    draw_context, fill_polygon, fill_rect, get_design_height2, get_drawable_width, get_left_inset_logical, scene_ignores_insets, set_draw_scale,
    set_insets_ignored, set_tint, set_tint_alpha, AppContext,
};

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
        } else if frame == 0xc {
            0xff
        } else if frame as u32 > 0x18 {
            0
        } else {
            frame.wrapping_mul(-0x15).wrapping_add(0x210)
        };

        set_tint_alpha(draw_context(&mut ctx.draw)?, alpha);
        set_insets_ignored(ctx, 1)?;

        let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let width = get_drawable_width(ctx)?;
        let height = get_design_height2(ctx);

        fill_rect(draw_context(&mut ctx.draw)?, 0, top, width, height);
        set_insets_ignored(ctx, 0)?;

        return Ok(());
    }

    if closing != 1 {
        return Ok(());
    }

    set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);

    let frame = ctx.i32_at(AppContext::FADE_FRAME)?;

    if frame <= 0xa {
        for (start, limit, lift, tall) in [(-0x180i32, 0x4ac, 0x37, 0x6e), (-0x112, 0x51a, 0x18, 0x30)] {
            let mut row = start;

            while row != limit {
                let width = get_drawable_width(ctx)?;
                let sweep = ctx.i32_at(AppContext::FADE_FRAME)?;
                let inset = if scene_ignores_insets(ctx)? != 0 { 0 } else { 0i32.wrapping_sub(get_left_inset_logical(ctx)) };
                let base = ops::div_neg_10(get_drawable_width(ctx)?.wrapping_mul(sweep)).wrapping_add(width).wrapping_add(inset);
                let notch = if tall == 0x6e { -0x6e } else { -0x37 };

                ctx.set_i32_at(AppContext::POLYGON_XS, base.wrapping_add(notch))?;
                ctx.set_i32_at(AppContext::POLYGON_YS, row.wrapping_add(lift))?;
                ctx.set_i32_at(AppContext::POLYGON_XS + 4, base)?;
                ctx.set_i32_at(AppContext::POLYGON_YS + 4, row)?;
                ctx.set_i32_at(AppContext::POLYGON_XS + 8, base)?;
                ctx.set_i32_at(AppContext::POLYGON_YS + 8, row.wrapping_add(tall))?;

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

                row += 0x9e;
            }
        }

        set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);

        let width = get_drawable_width(ctx)?;
        let sweep = ctx.i32_at(AppContext::FADE_FRAME)?;
        let inset = if scene_ignores_insets(ctx)? != 0 { 0 } else { 0i32.wrapping_sub(get_left_inset_logical(ctx)) };
        let x = ops::div_neg_10(get_drawable_width(ctx)?.wrapping_mul(sweep)).wrapping_add(width).wrapping_add(inset);
        let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let span = get_drawable_width(ctx)?.wrapping_mul(2);
        let height = get_design_height2(ctx);

        fill_rect(draw_context(&mut ctx.draw)?, x, top, span, height);

        return Ok(());
    }

    if (frame as u32) < 0xd {
        set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);
        set_insets_ignored(ctx, 1)?;

        let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let width = get_drawable_width(ctx)?;
        let height = get_design_height2(ctx);

        fill_rect(draw_context(&mut ctx.draw)?, 0, top, width, height);
        set_insets_ignored(ctx, 0)?;

        return Ok(());
    }

    for (start, limit, lift, tall, notch) in [(-0x180i32, 0x4ac, 0x37, 0x6e, 0xdc), (-0x112, 0x51a, 0x18, 0x30, 0xa5)] {
        let mut row = start;

        while row != limit {
            let width = get_drawable_width(ctx)?;
            let sweep = ctx.i32_at(AppContext::FADE_FRAME)?.wrapping_add(-1);
            let base = ops::div_neg_10(get_drawable_width(ctx)?.wrapping_mul(sweep))
                .wrapping_add(width)
                .wrapping_add(width);

            ctx.set_i32_at(AppContext::POLYGON_XS, base.wrapping_add(notch))?;
            ctx.set_i32_at(AppContext::POLYGON_YS, row.wrapping_add(lift))?;
            ctx.set_i32_at(AppContext::POLYGON_XS + 4, base.wrapping_add(0x6e))?;
            ctx.set_i32_at(AppContext::POLYGON_YS + 4, row)?;
            ctx.set_i32_at(AppContext::POLYGON_XS + 8, base.wrapping_add(0x6e))?;
            ctx.set_i32_at(AppContext::POLYGON_YS + 8, row.wrapping_add(tall))?;

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

            row += 0x9e;
        }
    }

    set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);

    let width = get_drawable_width(ctx)?;
    let sweep = ctx.i32_at(AppContext::FADE_FRAME)?.wrapping_add(-1);
    let x = ops::div_neg_10(get_drawable_width(ctx)?.wrapping_mul(sweep)).wrapping_add(width);
    let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
    let span = get_drawable_width(ctx)?.wrapping_add(0x6e);
    let height = get_design_height2(ctx);

    fill_rect(draw_context(&mut ctx.draw)?, x, top, span, height);

    Ok(())
}
