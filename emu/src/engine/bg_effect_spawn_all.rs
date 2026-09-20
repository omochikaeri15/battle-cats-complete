use std::collections::BTreeMap;

use crate::{Fault, ops};

use super::{
    AppContext, BgEffectRolls, bg_effect_roll_params, bg_param_roll_int, get_background_id,
    get_drawable_width, load_bg_effect_json,
};

pub fn bg_effect_spawn_all(ctx: &mut AppContext) -> Result<(), Fault> {
    let length = ctx.i32_at(AppContext::STAGE_LENGTH)?;
    let visible = ops::div_1000(
        ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?
            .wrapping_mul(length),
    );
    let span = get_drawable_width(ctx)?
        .wrapping_sub(visible)
        .wrapping_mul(length);
    let span = ops::idiv(span, visible).ok_or(Fault::divide(visible as i64))?;
    let half = ops::div_2(span);
    let left = ops::div_neg_20(span) as f64;
    let left = ((get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64) * 0.5 + left) as f32;
    let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
    let lift = ops::div_2(shift)
        .wrapping_mul(100)
        .wrapping_add(0xc350);
    let zoom = ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?;
    let lift = ops::idiv(lift, zoom).ok_or(Fault::divide(zoom as i64))?;
    let top = 0x1e0i32.wrapping_sub(lift) as f32;
    let centre = ((get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64) * 0.5) as f32;
    let scale = ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?.wrapping_mul(100) as f32 / 10000.0;
    let zero = 0.0f32 * scale;
    let back = ((0x3c0i32.wrapping_sub(get_drawable_width(ctx)?) as f64) * 0.5) as f32;
    let across = back * scale + zero * -520.0 + centre;
    let down = back * zero + -520.0 * scale + 520.0;
    let x = (left * scale + zero * top + across) * 1000.0
        / ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)? as f32
        + half as f32;
    let y = (-(zero * left + scale * top + down)).floor();

    ctx.bg_effects.origin_x = ops::cvttss2si(x);
    ctx.bg_effects.origin_y = ops::cvttss2si(y);
    ctx.bg_effects.defs.clear();

    let background = get_background_id(ctx)?;

    load_bg_effect_json(ctx, background)?;
    ctx.bg_effects.instances.clear();

    let mut groups: BTreeMap<i32, i32> = BTreeMap::new();
    let mut def = 0usize;

    while def < ctx.bg_effects.defs.len() {
        let spec = ctx.bg_effects.defs[def].count.clone();
        let mut count = bg_param_roll_int(ctx, &spec, &mut groups, -1, 0)?;

        if count > 0 {
            loop {
                ctx.bg_effects.instances.push(BgEffectRolls::default());

                let instance = ctx.bg_effects.instances.len() - 1;

                ctx.bg_effects.instances[instance].def_index = def as i32;
                bg_effect_roll_params(ctx, instance, 1)?;
                count = count.wrapping_sub(1);

                if count == 0 {
                    break;
                }
            }
        }

        def += 1;
    }

    Ok(())
}
