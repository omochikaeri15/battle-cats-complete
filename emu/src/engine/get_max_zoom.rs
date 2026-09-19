use crate::{Fault, operation};

use super::{AppContext, camera_vertical_correction, get_setting};

const SITE: &str = "get_max_zoom";

pub fn get_max_zoom(ctx: &mut AppContext) -> Result<i32, Fault> {
    let saved_zoom = ctx.i32_at(AppContext::CAMERA_ZOOM)?;
    let zoom_y = ctx.i32_at(AppContext::BATTLE_ZOOM_Y)?;

    ctx.set_i32_at(AppContext::CAMERA_ZOOM, 0x2710)?;

    let castle_y = get_setting(&ctx.settings, b"battle_zoom_castle_y", 0)?;
    let mut correction = 0i32;

    if ctx.u8_at(AppContext::DECK_TWO_LINES)? != 0 {
        correction = camera_vertical_correction(ctx)?;
    }

    let zoom_y_again = ctx.i32_at(AppContext::BATTLE_ZOOM_Y)?;
    let letterbox = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;

    let dividend = castle_y
        .wrapping_sub(zoom_y)
        .wrapping_mul(0x2710)
        .wrapping_add(0x11da50);
    let divisor = correction
        .wrapping_sub(zoom_y_again)
        .wrapping_add(letterbox)
        .wrapping_add(0x8a);

    let quotient = operation::idiv(dividend, divisor).ok_or(Fault::divide(SITE, divisor as i64))?;
    let max_zoom = if quotient < 0x2710 { quotient } else { 0x2710 };

    ctx.set_i32_at(AppContext::CAMERA_ZOOM, saved_zoom)?;

    Ok(max_zoom)
}
