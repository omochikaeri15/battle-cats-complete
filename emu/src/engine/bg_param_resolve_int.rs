use crate::{Fault, operation};

use super::{AppContext, get_design_height2, get_drawable_width, maanim_get_max_keyframe};

pub fn bg_param_resolve_int(
    ctx: &mut AppContext,
    reference: i32,
    value: i32,
    base: i32,
) -> Result<i32, Fault> {
    let resolved = match base.wrapping_sub(1) as u32 {
        0 => value.wrapping_mul(0x3c0),
        1 => operation::cvttsd2si(value as f64 + ctx.bg_effects.origin_x as f64 / -10.0),
        2 => operation::cvttss2si(
            value as f32
                + ctx
                    .bg_effects
                    .origin_x
                    .wrapping_add(ctx.i32_at(AppContext::STAGE_LENGTH)?) as f32
                    / 10.0,
        ),
        3 => {
            let lift = operation::div_2(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?)
                .wrapping_mul(0x64)
                .wrapping_add(0xc350);
            let min_zoom = ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?;
            let lift =
                operation::idiv(lift, min_zoom).ok_or(Fault::divide(min_zoom as i64))?;

            value
                .wrapping_sub(lift)
                .wrapping_add(ctx.bg_effects.origin_y)
                .wrapping_add(0x1e0)
        }
        4 => {
            let shifted = value.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
            let span = get_design_height2(ctx)
                .wrapping_mul(0x64)
                .wrapping_sub(0xcb20);
            let min_zoom = ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?;
            let span =
                operation::idiv(span, min_zoom).ok_or(Fault::divide(min_zoom as i64))?;

            shifted.wrapping_add(span).wrapping_add(0x208)
        }
        6 => get_drawable_width(ctx)?.wrapping_add(value),
        8 => get_design_height2(ctx).wrapping_add(value),
        10 => value.wrapping_add(1),
        11 => value.wrapping_add(2),
        12 => operation::div_30(value),
        13 => operation::div_100(value),
        14 => operation::div_100(value.wrapping_shl(8).wrapping_sub(value)),
        15 | 16 => {
            let file = ctx
                .bg_effects
                .model_anims
                .entry(reference)
                .or_default()
                .clone();
            let anim = ctx.bg_anim_cache.entry(file).or_default();

            value.wrapping_add(maanim_get_max_keyframe(anim)?)
        }
        _ => value,
    };

    Ok(resolved)
}
