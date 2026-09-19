use crate::Fault;

use super::{AppContext, get_map_type, labyrinth_active};

pub fn powerups_cleared(ctx: &mut AppContext) -> Result<bool, Fault> {
    if get_map_type(ctx, 0)? == -6 {
        return Ok(false);
    }

    let mode = if ctx.u8_at(AppContext::SCORE_MODE_FLAG)? != 0 {
        labyrinth_active(ctx)? as usize + 1
    } else {
        0
    };

    Ok(ctx.u8_at(AppContext::POWERUP_CLEARED + mode)? != 0)
}
