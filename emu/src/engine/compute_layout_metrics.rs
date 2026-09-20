use crate::{Fault, operation};

use super::{
    AppContext, get_scene_id, get_screen_height, get_screen_width, get_usable_height, has_insets,
};

pub fn compute_layout_metrics(ctx: &mut AppContext) -> Result<(), Fault> {
    let ratio = ctx
        .platform()
        .ok_or(Fault::host_missing())?
        .screen_window_ratio();

    ctx.screen_metrics.window_ratio = ratio;
    ctx.screen_metrics.scale2 = 1.0;
    ctx.screen_metrics.screen_w = get_screen_width(ctx);
    ctx.screen_metrics.screen_h = get_screen_height(ctx);

    let width = ctx.screen_metrics.screen_w;
    let height = ctx.screen_metrics.screen_h;

    ctx.screen_metrics.screen_w2 = width;
    ctx.screen_metrics.screen_h2 = height;

    let tall = 960.0 * height as f32;
    let design_w;
    let design_h;

    if (width as f32) < tall / 640.0 {
        ctx.screen_metrics.design_w = 0x3c0;

        let scaled = operation::lroundf(tall / width as f32) as i32;

        ctx.screen_metrics.design_h = scaled;
        ctx.screen_metrics.design_h2 = scaled;
        design_w = 0x3c0;
        design_h = scaled;
    } else {
        ctx.screen_metrics.design_w =
            operation::lroundf(width as f32 * 640.0 / height as f32) as i32;
        ctx.screen_metrics.design_h2 = 0x280;
        ctx.screen_metrics.design_h = 0x280;

        let left = ctx
            .platform()
            .ok_or(Fault::host_missing())?
            .safe_inset_left();
        let right = ctx
            .platform()
            .ok_or(Fault::host_missing())?
            .safe_inset_right();

        ctx.screen_metrics.inset_left = if left > right { left } else { right };
        ctx.screen_metrics.inset_top = ctx
            .platform()
            .ok_or(Fault::host_missing())?
            .safe_inset_top();
        ctx.screen_metrics.inset_right = ctx.screen_metrics.inset_left;
        ctx.screen_metrics.inset_bottom = ctx
            .platform()
            .ok_or(Fault::host_missing())?
            .safe_inset_bottom();

        if has_insets(ctx) {
            let scaled = ctx.screen_metrics.screen_h as f32 * ctx.screen_metrics.design_w as f32;

            ctx.screen_metrics.design_w = operation::lroundf(
                scaled / get_usable_height(&ctx.screen_metrics) as f32,
            ) as i32;

            let scaled = ctx.screen_metrics.screen_h as f32 * ctx.screen_metrics.design_h2 as f32;
            let height2 = operation::lroundf(
                scaled / get_usable_height(&ctx.screen_metrics) as f32,
            ) as i32;

            ctx.screen_metrics.design_h2 = height2;
            design_h = height2;
            design_w = ctx.screen_metrics.design_w;

            if design_w >= 0x51d {
                let pad = operation::lroundf(
                    (design_w.wrapping_sub(0x51c) as f32) * 0.5 * ctx.screen_metrics.window_ratio,
                ) as i32;
                let widened = pad.wrapping_add(ctx.screen_metrics.inset_left);

                ctx.screen_metrics.inset_left = widened;
                ctx.screen_metrics.inset_right = widened;
            }
        } else {
            design_w = ctx.screen_metrics.design_w;
            design_h = ctx.screen_metrics.design_h2;
        }
    }

    let width = ctx.screen_metrics.screen_w;
    let height = ctx.screen_metrics.screen_h;
    let fitted = operation::idiv(width.wrapping_mul(design_h), design_w)
        .ok_or(Fault::divide(design_w as i64))?;
    let span = if fitted > height {
        operation::lroundf(height as f32 * design_w as f32 / design_h as f32) as f32
    } else {
        width as f32
    };

    ctx.screen_metrics.scale2 = (span + span) / design_w as f32;

    let shift = operation::lroundf(
        (ctx.screen_metrics.design_h.wrapping_sub(0x280) as f32) * 0.5,
    ) as i32;

    ctx.set_i32_at(AppContext::LETTERBOX_SHIFT, shift)?;

    let mut pad = 0i32;

    if ctx.i32_at(AppContext::LETTERBOX_SHIFT)? > 0 && ctx.screen_metrics.design_h >= 0x2d0 {
        ctx.set_i32_at(AppContext::LETTERBOX_SHIFT, 0x28)?;

        let scaled = ctx.screen_metrics.screen_h as f32 * ctx.screen_metrics.design_w as f32;
        let logical = operation::lroundf(scaled / ctx.screen_metrics.screen_w as f32);

        pad = operation::lround(
            operation::div_2(logical.wrapping_sub(0x2d0)) as f64,
        ) as i32;
    }

    ctx.set_i32_at(AppContext::LETTERBOX_PAD, pad)?;

    let ratio = ctx.screen_metrics.window_ratio;

    ctx.ui()
        .ok_or(Fault::host_missing())?
        .set_window_ratio(ratio);

    if get_scene_id(ctx)? == 0x64 && ctx.i32_at(AppContext::SCENE_0X64_PAGE)? == 0 {
        ctx.ui()
            .ok_or(Fault::host_missing())?
            .clear_layout_latch();
        ctx.ui()
            .ok_or(Fault::host_missing())?
            .set_layout_latch();
    }

    ctx.ui()
        .ok_or(Fault::host_missing())?
        .viewport_resized();

    Ok(())
}
