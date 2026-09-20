use crate::{Fault, ops};

use super::{
    AppContext, cos_deg, draw_context, draw_cut_f, draw_cut_rotated, draw_cut_rotated_f,
    draw_cut_scaled, get_base_pos_x, get_base_pos_y, get_drawable_width, get_powerup, set_flip,
    set_tint, sin_deg,
};

pub fn sniper_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    if !get_powerup(ctx, 5)? {
        return Ok(());
    }

    let base_x =
        ops::div_10(get_base_pos_x(ctx, 0)?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?))
            .wrapping_add(0x64);

    ctx.set_i32_at(AppContext::DRAW_TEMP_1, base_x)?;

    let base_y = ops::div_10(get_base_pos_y(ctx, 0)?).wrapping_add(-0x2ab) as f32;
    let bob = sin_deg(ctx.f32_at(AppContext::SNIPER_BOB_ANGLE)?) * 10.0 + base_y;

    ctx.set_i32_at(AppContext::DRAW_TEMP_2, ops::cvttss2si(bob))?;

    let origin = ctx
        .i32_at(AppContext::CAMERA_KICK)?
        .wrapping_add(ctx.i32_at(AppContext::DRAW_TEMP_1)?) as f64;
    let x = (get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin) as f32;
    let y = ctx.i32_at(AppContext::DRAW_TEMP_2)? as f32 + 114.56;

    draw_cut_f(
        draw_context(&mut ctx.draw)?,
        ctx.img043_sheet
            .as_deref()
            .ok_or(Fault::null_pointer())?,
        6,
        x,
        y,
        53.7,
        161.1,
    );
    set_flip(draw_context(&mut ctx.draw)?, 1);

    let origin = ctx.i32_at(AppContext::CAMERA_KICK)? as f32
        + (ctx.i32_at(AppContext::DRAW_TEMP_1)? as f32 + 132.46);
    let x = (get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin as f64) as f32;
    let y = ctx.i32_at(AppContext::DRAW_TEMP_2)? as f32 + 114.56;

    draw_cut_f(
        draw_context(&mut ctx.draw)?,
        ctx.img043_sheet
            .as_deref()
            .ok_or(Fault::null_pointer())?,
        6,
        x,
        y,
        53.7,
        161.1,
    );
    set_flip(draw_context(&mut ctx.draw)?, 0);

    let firing = ctx.u8_at(AppContext::SNIPER_FIRING)?;
    let left = ctx.i32_at(AppContext::DRAW_TEMP_1)? as f32;
    let kick = ctx.i32_at(AppContext::CAMERA_KICK)? as f32;
    let half = get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64;
    let y = ctx.i32_at(AppContext::DRAW_TEMP_2)? as f32 + 202.27;

    if firing != 0 {
        let x = (half * 0.5 + (left + 53.7 + kick) as f64) as f32;
        let y = y + ctx.i32_at(AppContext::SNIPER_RECOIL)? as f32;

        draw_cut_f(
            draw_context(&mut ctx.draw)?,
            ctx.img043_sheet
                .as_deref()
                .ok_or(Fault::null_pointer())?,
            5,
            x,
            y,
            82.34,
            91.29,
        );
    } else {
        let x = (half * 0.5 + (left + 51.91 + kick) as f64) as f32;

        draw_cut_f(
            draw_context(&mut ctx.draw)?,
            ctx.img043_sheet
                .as_deref()
                .ok_or(Fault::null_pointer())?,
            4,
            x,
            y,
            82.34,
            91.29,
        );
    }

    let origin = ctx.i32_at(AppContext::CAMERA_KICK)? as f32
        + (ctx.i32_at(AppContext::DRAW_TEMP_1)? as f32 + 42.96);
    let x = (get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin as f64) as f32;
    let y = ctx.i32_at(AppContext::DRAW_TEMP_2)? as f32 + 286.4;

    draw_cut_f(
        draw_context(&mut ctx.draw)?,
        ctx.img043_sheet
            .as_deref()
            .ok_or(Fault::null_pointer())?,
        3,
        x,
        y,
        91.29,
        42.96,
    );

    let origin = ctx.i32_at(AppContext::CAMERA_KICK)? as f32
        + (ctx.i32_at(AppContext::DRAW_TEMP_1)? as f32 + 34.01);
    let x = (get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin as f64) as f32;
    let y = ctx.i32_at(AppContext::DRAW_TEMP_2)? as f32 + 288.19;
    let angle = ctx.f32_at(AppContext::SNIPER_AIM_ANGLE)?;

    draw_cut_rotated_f(
        draw_context(&mut ctx.draw)?,
        ctx.img043_sheet
            .as_deref()
            .ok_or(Fault::null_pointer())?,
        x,
        y,
        93.08,
        57.28,
        68.0,
        26.0,
        angle,
        0,
        0,
        2,
    );
    set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);

    let origin = ctx.i32_at(AppContext::CAMERA_KICK)? as f32
        + (ctx.i32_at(AppContext::DRAW_TEMP_1)? as f32 + 34.01);
    let x = (get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin as f64) as f32;
    let y = ctx.i32_at(AppContext::DRAW_TEMP_2)? as f32 + 268.5;

    draw_cut_f(
        draw_context(&mut ctx.draw)?,
        ctx.img043_sheet
            .as_deref()
            .ok_or(Fault::null_pointer())?,
        1,
        x,
        y,
        118.14,
        59.07,
    );

    let origin = ctx
        .i32_at(AppContext::CAMERA_KICK)?
        .wrapping_add(ctx.i32_at(AppContext::DRAW_TEMP_1)?) as f64;
    let x = (get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin) as f32;
    let y = ctx.i32_at(AppContext::DRAW_TEMP_2)? as f32;

    draw_cut_f(
        draw_context(&mut ctx.draw)?,
        ctx.img043_sheet
            .as_deref()
            .ok_or(Fault::null_pointer())?,
        0,
        x,
        y,
        93.08,
        187.95,
    );
    set_flip(draw_context(&mut ctx.draw)?, 1);

    let origin = ctx.i32_at(AppContext::CAMERA_KICK)? as f32
        + (ctx.i32_at(AppContext::DRAW_TEMP_1)? as f32 + 93.08);
    let x = (get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin as f64) as f32;
    let y = ctx.i32_at(AppContext::DRAW_TEMP_2)? as f32;

    draw_cut_f(
        draw_context(&mut ctx.draw)?,
        ctx.img043_sheet
            .as_deref()
            .ok_or(Fault::null_pointer())?,
        0,
        x,
        y,
        93.08,
        187.95,
    );
    set_flip(draw_context(&mut ctx.draw)?, 0);

    let firing = ctx.u8_at(AppContext::SNIPER_FIRING)?;
    let left = ctx.i32_at(AppContext::DRAW_TEMP_1)? as f32;
    let kick = ctx.i32_at(AppContext::CAMERA_KICK)? as f32;
    let half = get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64;
    let top = ctx.i32_at(AppContext::DRAW_TEMP_2)? as f32;

    if firing != 0 {
        let x = (half * 0.5 + (left + 51.91 + kick) as f64) as f32;
        let y = top + 186.16 + ctx.i32_at(AppContext::SNIPER_RECOIL)? as f32;

        draw_cut_f(
            draw_context(&mut ctx.draw)?,
            ctx.img043_sheet
                .as_deref()
                .ok_or(Fault::null_pointer())?,
            9,
            x,
            y,
            89.5,
            69.81,
        );
    } else {
        let x = (half * 0.5 + (left + 50.12 + kick) as f64) as f32;
        let y = top + 193.32;

        draw_cut_f(
            draw_context(&mut ctx.draw)?,
            ctx.img043_sheet
                .as_deref()
                .ok_or(Fault::null_pointer())?,
            9,
            x,
            y,
            89.5,
            69.81,
        );
    }

    let reach = cos_deg(ctx.f32_at(AppContext::SNIPER_AIM_ANGLE)?) * 47.0;

    ctx.set_i32_at(AppContext::DRAW_TEMP_3, ops::cvttss2si(reach))?;

    let lift = sin_deg(ctx.f32_at(AppContext::SNIPER_AIM_ANGLE)?) * 28.0;

    ctx.set_i32_at(AppContext::DRAW_TEMP_4, ops::cvttss2si(lift))?;

    if ctx.u8_at(AppContext::SNIPER_CASINGS_LIVE)? != 0 {
        for casing in 0..4usize {
            let base = AppContext::SNIPER_CASINGS + casing * 0x14;
            let origin = ctx
                .i32_at(AppContext::DRAW_TEMP_1)?
                .wrapping_add(ctx.i32_at(AppContext::CAMERA_KICK)?)
                .wrapping_sub(ctx.i32_at(AppContext::DRAW_TEMP_3)?)
                .wrapping_add(ctx.i32_at(base)?)
                .wrapping_add(0x23) as f64;
            let x = ops::cvttsd2si(
                get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin,
            );
            let y = ctx
                .i32_at(AppContext::DRAW_TEMP_2)?
                .wrapping_sub(ctx.i32_at(AppContext::DRAW_TEMP_4)?)
                .wrapping_add(ctx.i32_at(base + 4)?)
                .wrapping_add(0x100);
            let cut = ops::div_2(ctx.i32_at(base + 8)?).wrapping_add(8);

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

    let base_x =
        ops::div_10(get_base_pos_x(ctx, 0)?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?))
            .wrapping_add(0x99);

    ctx.set_i32_at(AppContext::DRAW_TEMP_1, base_x)?;

    let base_y = ops::div_10(get_base_pos_y(ctx, 0)?).wrapping_add(0x2f);

    ctx.set_i32_at(AppContext::DRAW_TEMP_2, base_y)?;

    let origin = ctx
        .i32_at(AppContext::CAMERA_KICK)?
        .wrapping_add(ctx.i32_at(AppContext::DRAW_TEMP_1)?) as f64;
    let x = (get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin) as f32;
    let y = ctx.i32_at(AppContext::DRAW_TEMP_2)? as f32;

    draw_cut_f(
        draw_context(&mut ctx.draw)?,
        ctx.img043_sheet
            .as_deref()
            .ok_or(Fault::null_pointer())?,
        7,
        x,
        y,
        80.55,
        7.16,
    );

    for strike in 0..0x32usize {
        if ctx.u8_at(AppContext::PENDING_STRIKE_ACTIVE + strike)? == 0 {
            continue;
        }

        let pos_x = ops::div_10(
            ctx.i32_at(AppContext::PENDING_STRIKE_TRIGGER_X + strike * 4)?
                .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?),
        );

        ctx.set_i32_at(AppContext::DRAW_TEMP_1, pos_x)?;

        let x = ops::cvttsd2si(
            get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + pos_x as f64,
        );
        let y = ops::div_10(ctx.i32_at(AppContext::PENDING_STRIKE_Y + strike * 4)?);

        draw_cut_rotated(
            draw_context(&mut ctx.draw)?,
            ctx.img043_sheet
                .as_deref()
                .ok_or(Fault::null_pointer())?,
            x,
            y,
            0x35,
            0x35,
            0.0,
            0,
            0,
            0,
            5,
            8,
        );
    }

    for spark in 0..0x32usize {
        let base =
            AppContext::PENDING_STRIKE_SPARKS + spark * AppContext::PENDING_STRIKE_SPARKS_STRIDE;
        let timer = ctx.i32_at(base)?;

        if timer > 0 {
            let pos_x = ops::div_10(
                ctx.i32_at(base + 4)?
                    .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?),
            );

            ctx.set_i32_at(AppContext::DRAW_TEMP_1, pos_x)?;

            if timer != 1 {
                let x = ops::cvttsd2si(
                    get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + pos_x as f64,
                );
                let y = ops::div_10(ctx.i32_at(base + 8)?);
                let cut = 0xdi32.wrapping_sub(ops::div_2(ctx.i32_at(base)?));

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

        if ctx.i32_at(base + 0xc)? > 0 {
            let pos_x = ops::div_10(
                ctx.i32_at(base + 0x10)?
                    .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?),
            );

            ctx.set_i32_at(AppContext::DRAW_TEMP_1, pos_x)?;

            if ctx.i32_at(base + 0xc)? >= 2 {
                let x = ops::cvttsd2si(
                    get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + pos_x as f64,
                );
                let y = ops::div_10(ctx.i32_at(base + 0x14)?);
                let cut = 0xdi32.wrapping_sub(ops::div_2(ctx.i32_at(base + 0xc)?));

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
    }

    Ok(())
}
