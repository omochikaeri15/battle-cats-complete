use crate::{Fault, operation};

use super::{AppContext, get_design_height2, get_drawable_width, maanim_get_max_keyframe};

pub fn bg_param_resolve_float(
    ctx: &mut AppContext,
    reference: i32,
    value: f32,
    base: i32,
) -> Result<f32, Fault> {
    let resolved = match base.wrapping_sub(1) as u32 {
        0 => value * 960.0,
        1 => (value as f64 + ctx.bg_effects.origin_x as f64 / -10.0) as f32,
        2 => {
            value
                + ctx
                    .bg_effects
                    .origin_x
                    .wrapping_add(ctx.i32_at(AppContext::STAGE_LENGTH)?) as f32
                    / 10.0
        }
        3 => {
            let lift = operation::div_2(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?)
                .wrapping_mul(0x64)
                .wrapping_add(0xc350);
            let min_zoom = ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?;
            let lift =
                operation::idiv(lift, min_zoom).ok_or(Fault::divide(min_zoom as i64))?;

            value
                + ctx
                    .bg_effects
                    .origin_y
                    .wrapping_sub(lift)
                    .wrapping_add(0x1e0) as f32
        }
        4 => {
            let letterbox = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
            let span = get_design_height2(ctx)
                .wrapping_mul(0x64)
                .wrapping_sub(0xcb20);
            let min_zoom = ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?;
            let span =
                operation::idiv(span, min_zoom).ok_or(Fault::divide(min_zoom as i64))?;

            value + span.wrapping_sub(letterbox).wrapping_add(0x208) as f32
        }
        6 => value + get_drawable_width(ctx)? as f32,
        8 => value + get_design_height2(ctx) as f32,
        10 => value + 1.0,
        11 => value + 2.0,
        12 => value / 30.0,
        13 => value / 100.0,
        14 => value * 255.0 / 100.0,
        15 | 16 => {
            let file = ctx
                .bg_effects
                .model_anims
                .entry(reference)
                .or_default()
                .clone();
            let anim = ctx.bg_anim_cache.entry(file).or_default();

            value + maanim_get_max_keyframe(anim)? as f32
        }
        _ => value,
    };

    Ok(resolved)
}
