use crate::Fault;

use super::{AppContext, labyrinth_active};

pub fn get_powerup_available(ctx: &AppContext) -> Result<u8, Fault> {
    let mut mode = 0usize;

    if ctx.u8_at(AppContext::SCORE_MODE_FLAG)? != 0 {
        mode = labyrinth_active(ctx)? as usize + 1;
    }

    ctx.u8_at(AppContext::POWERUP_AVAILABLE.wrapping_add(mode))
}
