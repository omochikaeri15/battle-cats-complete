use crate::Fault;

use super::{AppContext, labyrinth_active};

pub fn set_powerup_available(ctx: &mut AppContext, value: u8) -> Result<(), Fault> {
    let mode = if ctx.u8_at(AppContext::SCORE_MODE_FLAG)? != 0 {
        labyrinth_active(ctx)? as usize + 1
    } else {
        0
    };

    ctx.set_block_at::<1>(AppContext::POWERUP_AVAILABLE + mode, [value])
}
