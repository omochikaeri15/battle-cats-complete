use crate::{Fault, operation};

use super::{
    AppContext, Debris, draw_context, draw_cut_scaled, get_drawable_width, glow_set, set_color,
};

pub fn draw_smoke(ctx: &mut AppContext) -> Result<(), Fault> {
    let mut group = 0usize;
    let mut first = true;

    loop {
        for record in 0..0x38usize {
            let base = AppContext::CAT_DEBRIS + group * 0x380 + record * 0x10;

            if ctx.i32_at(base + Debris::TIMER)? <= 0 {
                continue;
            }

            let variant = ctx.i32_at(base + Debris::VARIANT)?;

            glow_set(draw_context(&mut ctx.draw)?, 0);

            if variant == 2 || variant == 1 {
                if variant == 2 {
                    set_color(draw_context(&mut ctx.draw)?, 0xc8, 0xaa, 0xff, 0xff);
                } else {
                    set_color(draw_context(&mut ctx.draw)?, 0xcc, 0xcc, 0xcc, 0xff);
                }

                let pos_x = operation::div_10(
                    ctx.i32_at(base + Debris::POS_X)?
                        .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?),
                );

                ctx.set_i32_at(AppContext::DRAW_TEMP_1, pos_x)?;

                if ctx.i32_at(base + Debris::TIMER)? >= 2 {
                    let x = operation::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0))
                        .wrapping_add(pos_x);
                    let y = operation::div_10(ctx.i32_at(base + Debris::POS_Y)?);
                    let cut =
                        0xdi32.wrapping_sub(operation::div_2(ctx.i32_at(base + Debris::TIMER)?));

                    draw_cut_scaled(
                        draw_context(&mut ctx.draw)?,
                        ctx.effect_a_sheet
                            .as_deref()
                            .ok_or(Fault::null_pointer())?,
                        x,
                        y,
                        0x89,
                        0x75,
                        cut,
                    );
                }

                glow_set(draw_context(&mut ctx.draw)?, 1);

                if variant == 2 {
                    set_color(draw_context(&mut ctx.draw)?, 0x91, 0x87, 0xa5, 0xff);
                } else {
                    set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);
                }
            } else {
                set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);
            }

            let pos_x = operation::div_10(
                ctx.i32_at(base + Debris::POS_X)?
                    .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?),
            );

            ctx.set_i32_at(AppContext::DRAW_TEMP_1, pos_x)?;

            if ctx.i32_at(base + Debris::TIMER)? >= 2 {
                let x = operation::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0))
                    .wrapping_add(pos_x);
                let y = operation::div_10(ctx.i32_at(base + Debris::POS_Y)?);
                let cut = 0xdi32.wrapping_sub(operation::div_2(ctx.i32_at(base + Debris::TIMER)?));

                draw_cut_scaled(
                    draw_context(&mut ctx.draw)?,
                    ctx.effect_a_sheet
                        .as_deref()
                        .ok_or(Fault::null_pointer())?,
                    x,
                    y,
                    0x89,
                    0x75,
                    cut,
                );
            }
        }

        group = 1;

        if !first {
            break;
        }

        first = false;
    }

    glow_set(draw_context(&mut ctx.draw)?, 0);
    set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

    Ok(())
}
