use crate::{Fault, ops};

use super::{
    AppContext, bg_has_upper_layer, camera_vertical_correction, draw_context, draw_cut_scaled,
    fill_polygon_colored, fill_rect, get_base_shake_offset, get_bg_ground_bottom,
    get_bg_ground_top, get_bg_sky_bottom, get_bg_sky_top, get_design_height2, get_drawable_width,
    imgcut_get_sprite_cut, set_tint, set_transform,
};

pub fn draw_background(ctx: &mut AppContext) -> Result<(), Fault> {
    let horizon = camera_vertical_correction(ctx)?.wrapping_add(0x208);
    let color = get_bg_sky_bottom(ctx, AppContext::BG_SETUP)?;

    set_tint(
        draw_context(&mut ctx.draw)?,
        (color >> 0x10) & 0xff,
        (color >> 8) & 0xff,
        color & 0xff,
        0xff,
    );

    let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
    let width = get_drawable_width(ctx)?;

    fill_rect(draw_context(&mut ctx.draw)?, 0, top, width, horizon);

    if get_base_shake_offset(ctx) > 0 {
        let color = get_bg_sky_top(ctx, AppContext::BG_SETUP)?;

        set_tint(
            draw_context(&mut ctx.draw)?,
            (color >> 0x10) & 0xff,
            (color >> 8) & 0xff,
            color & 0xff,
            0xff,
        );

        let top = (-0x28i32).wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let width = get_drawable_width(ctx)?;
        let height = get_base_shake_offset(ctx);

        fill_rect(draw_context(&mut ctx.draw)?, 0, top, width, height);
    }

    let color = get_bg_ground_top(ctx, AppContext::BG_SETUP)?;

    set_tint(
        draw_context(&mut ctx.draw)?,
        (color >> 0x10) & 0xff,
        (color >> 8) & 0xff,
        color & 0xff,
        0xff,
    );

    let top = horizon.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
    let width = get_drawable_width(ctx)?;
    let height = get_design_height2(ctx).wrapping_sub(horizon);

    fill_rect(draw_context(&mut ctx.draw)?, 0, top, width, height);

    if get_base_shake_offset(ctx) < 0 {
        let color = get_bg_ground_bottom(ctx, AppContext::BG_SETUP)?;

        set_tint(
            draw_context(&mut ctx.draw)?,
            (color >> 0x10) & 0xff,
            (color >> 8) & 0xff,
            color & 0xff,
            0xff,
        );

        let top = ctx
            .i32_at(AppContext::LETTERBOX_SHIFT)?
            .wrapping_add(get_base_shake_offset(ctx))
            .wrapping_add(0x2a8);
        let width = get_drawable_width(ctx)?;
        let height = get_base_shake_offset(ctx).wrapping_neg();

        fill_rect(draw_context(&mut ctx.draw)?, 0, top, width, height);
    }

    let sheet = ctx.bg_sheet.clone();
    let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
    let scenery = ops::div_255(imgcut_get_sprite_cut(sheet, 0)?[3].wrapping_mul(0x280));

    ctx.set_block_at::<8>(AppContext::BG_TINT_XS, [0; 8])?;

    let width = get_drawable_width(ctx)?;

    ctx.set_i32_at(AppContext::BG_TINT_XS + 0xc, width)?;
    ctx.set_i32_at(AppContext::BG_TINT_XS + 8, width)?;

    let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
    let top = get_base_shake_offset(ctx)
        .wrapping_sub(shift)
        .wrapping_add(-0x28);

    ctx.set_i32_at(AppContext::BG_TINT_YS + 0xc, top)?;
    ctx.set_i32_at(AppContext::BG_TINT_YS, top)?;

    let anchor = ctx
        .i32_at(AppContext::BATTLE_ZOOM_Y)?
        .wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?) as f64;
    let zoom = ctx.i32_at(AppContext::CAMERA_ZOOM)? as f64 / 100.0;
    let lift = (100.0 - zoom) * anchor / 100.0;
    let base = if bg_has_upper_layer(ctx, AppContext::BG_SETUP)? != 0 {
        scenery
    } else {
        scenery.wrapping_add(-0x280)
    };
    let span = base
        .wrapping_sub(camera_vertical_correction(ctx)?)
        .wrapping_mul(ctx.i32_at(AppContext::CAMERA_ZOOM)?);
    let lift = lift - ops::div_10000(span) as f64;
    let bottom = ops::cvttsd2si(get_base_shake_offset(ctx) as f64 + lift);

    ctx.set_i32_at(AppContext::BG_TINT_YS + 8, bottom)?;
    ctx.set_i32_at(AppContext::BG_TINT_YS + 4, bottom)?;

    let color = get_bg_sky_top(ctx, AppContext::BG_SETUP)? | 0xff000000u32 as i32;

    ctx.set_i32_at(AppContext::BG_TINT_COLORS + 0xc, color)?;
    ctx.set_i32_at(AppContext::BG_TINT_COLORS, color)?;

    let color = get_bg_sky_bottom(ctx, AppContext::BG_SETUP)?;

    ctx.set_i32_at(AppContext::BG_TINT_COLORS + 8, color)?;
    ctx.set_i32_at(AppContext::BG_TINT_COLORS + 4, color)?;

    if ctx.i32_at(AppContext::BG_TINT_YS)? < ctx.i32_at(AppContext::BG_TINT_YS + 4)? {
        let mut xs = [0i32; 4];
        let mut ys = [0i32; 4];
        let mut colors = [0u32; 4];

        for corner in 0..4usize {
            xs[corner] = ctx.i32_at(AppContext::BG_TINT_XS + corner * 4)?;
            ys[corner] = ctx.i32_at(AppContext::BG_TINT_YS + corner * 4)?;
            colors[corner] = ctx.i32_at(AppContext::BG_TINT_COLORS + corner * 4)? as u32;
        }

        fill_polygon_colored(draw_context(&mut ctx.draw)?, &xs, &ys, &colors, 4);
    }

    ctx.set_block_at::<8>(AppContext::BG_TINT_XS, [0; 8])?;

    let width = get_drawable_width(ctx)?;

    ctx.set_i32_at(AppContext::BG_TINT_XS + 0xc, width)?;
    ctx.set_i32_at(AppContext::BG_TINT_XS + 8, width)?;

    let anchor = ctx
        .i32_at(AppContext::BATTLE_ZOOM_Y)?
        .wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?) as f64;
    let zoom = ctx.i32_at(AppContext::CAMERA_ZOOM)? as f64 / 100.0;
    let lift = (100.0 - zoom) * anchor / 100.0;
    let span = camera_vertical_correction(ctx)?
        .wrapping_add(0x280)
        .wrapping_mul(ctx.i32_at(AppContext::CAMERA_ZOOM)?);
    let lift = ops::div_10000(span) as f64 + lift;
    let top = ops::cvttsd2si(get_base_shake_offset(ctx) as f64 + lift);

    ctx.set_i32_at(AppContext::BG_TINT_YS + 0xc, top)?;
    ctx.set_i32_at(AppContext::BG_TINT_YS, top)?;

    let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
    let bottom = get_base_shake_offset(ctx)
        .wrapping_add(shift)
        .wrapping_add(0x2a8);

    ctx.set_i32_at(AppContext::BG_TINT_YS + 8, bottom)?;
    ctx.set_i32_at(AppContext::BG_TINT_YS + 4, bottom)?;

    let color = get_bg_ground_top(ctx, AppContext::BG_SETUP)?;

    ctx.set_i32_at(AppContext::BG_TINT_COLORS + 0xc, color)?;
    ctx.set_i32_at(AppContext::BG_TINT_COLORS, color)?;

    let color = get_bg_ground_bottom(ctx, AppContext::BG_SETUP)? | 0xff000000u32 as i32;

    ctx.set_i32_at(AppContext::BG_TINT_COLORS + 8, color)?;
    ctx.set_i32_at(AppContext::BG_TINT_COLORS + 4, color)?;

    let mut xs = [0i32; 4];
    let mut ys = [0i32; 4];
    let mut colors = [0u32; 4];

    for corner in 0..4usize {
        xs[corner] = ctx.i32_at(AppContext::BG_TINT_XS + corner * 4)?;
        ys[corner] = ctx.i32_at(AppContext::BG_TINT_YS + corner * 4)?;
        colors[corner] = ctx.i32_at(AppContext::BG_TINT_COLORS + corner * 4)? as u32;
    }

    fill_polygon_colored(draw_context(&mut ctx.draw)?, &xs, &ys, &colors, 4);

    let mut transform = [0.0f32; 6];

    for (index, value) in transform.iter_mut().enumerate() {
        *value = ctx.f32_at(AppContext::CAMERA_MATRIX + index * 4)?;
    }

    let scale = ctx.screen_metrics.scale2;

    set_transform(draw_context(&mut ctx.draw)?, scale, &transform);

    let camera = ops::div_10(ctx.i32_at(AppContext::CAMERA_X)?);
    let tiles = ops::div_960(camera).wrapping_mul(0x3c0);
    let origin = tiles.wrapping_sub(camera).wrapping_add(ops::div_2(
        get_drawable_width(ctx)?.wrapping_add(-0x3c0),
    ));
    let floor = 0x280i32.wrapping_sub(scenery);
    let mut offset = -0x780i32;

    while offset != 0xf00 {
        let x = origin.wrapping_add(offset);

        draw_cut_scaled(
            draw_context(&mut ctx.draw)?,
            sheet,
            x,
            floor,
            0x3c0,
            scenery,
            0,
        );

        if bg_has_upper_layer(ctx, AppContext::BG_SETUP)? != 0 {
            draw_cut_scaled(
                draw_context(&mut ctx.draw)?,
                sheet,
                x,
                -0x280,
                0x3c0,
                0x280,
                0x14,
            );
        }

        offset = offset.wrapping_add(0x3c0);
    }

    Ok(())
}
