use crate::{Fault, ops};

use super::{AppContext, get_setting};

pub fn camera_vertical_correction(ctx: &AppContext) -> Result<i32, Fault> {
    let slot_y = get_setting(&ctx.settings, b"battle_zoom_slot_y", 0x208)?;

    let zoom = ctx.i32_at(AppContext::CAMERA_ZOOM)?;
    let anchor = ctx
        .i32_at(AppContext::BATTLE_ZOOM_Y)?
        .wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

    let scaled = if ctx.u8_at(AppContext::DECK_TWO_LINES)? != 0 {
        get_setting(&ctx.settings, b"battle_slot_2lines_y", -0x28)?
            .wrapping_add(slot_y)
            .wrapping_mul(ctx.i32_at(AppContext::CAMERA_ZOOM)?)
    } else {
        slot_y.wrapping_mul(zoom)
    };

    let mut limit = ctx
        .deck_bar_base_y
        .wrapping_add(ctx.i32_at(AppContext::DECK_BAR_SLIDE)?);
    let mut correction = 0i32;

    limit = limit.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

    if ctx.u8_at(AppContext::DECK_TWO_LINES)? != 0 {
        let line = get_setting(&ctx.settings, b"battle_slot_2lines_line", 0x5a)?;

        limit = limit.wrapping_sub(line);

        if ctx.u8_at(AppContext::DECK_TWO_LINES)? != 0 {
            correction = get_setting(&ctx.settings, b"battle_slot_2lines_y", -0x28)?;
        }
    }

    let slack = (100.0f64 - (zoom as f64) / 100.0f64) * (anchor as f64) / 100.0f64;
    let bottom = ops::cvttsd2si((ops::div_10000(scaled as i64) as i32) as f64 + slack);

    if bottom > limit {
        let overshoot = bottom.wrapping_sub(limit).wrapping_mul(0x2710);
        let divisor = ctx.i32_at(AppContext::CAMERA_ZOOM)?;

        correction = correction.wrapping_sub(
            ops::idiv(overshoot, divisor).ok_or(Fault::divide(divisor as i64))?,
        );
    }

    Ok(correction)
}
