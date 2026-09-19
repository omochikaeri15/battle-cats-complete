use crate::Fault;

use super::AppContext;

pub fn get_preset_foundation_part(ctx: &AppContext) -> Result<i32, Fault> {
    let preset = ctx.i32_at(AppContext::SELECTED_DECK_PRESET)? as i64;

    Ok(ctx.i8_at((AppContext::PRESET_FOUNDATION_PARTS as i64).wrapping_add(preset.wrapping_mul(3)) as usize)? as i32)
}
