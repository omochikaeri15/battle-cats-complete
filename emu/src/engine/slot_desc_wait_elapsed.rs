use crate::Fault;

use super::{AppContext, get_setting};

pub fn slot_desc_wait_elapsed(ctx: &AppContext) -> Result<bool, Fault> {
    if ctx.i32_at(AppContext::DECK_HOLD_SLOT)? == -1 {
        return Ok(false);
    }

    let frames = ctx.i32_at(AppContext::DECK_HOLD_FRAMES)?;

    Ok(frames >= get_setting(&ctx.settings, b"battle_slot_desc_wait", 6)?)
}
