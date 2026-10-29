use crate::Fault;

use super::{AppContext, get_map_type, labyrinth_active};

pub fn replay_mode(ctx: &mut AppContext) -> Result<bool, Fault> {
    if ctx.u8_at(AppContext::REPLAY_MODE)? == 0 {
        return Ok(false);
    }

    if get_map_type(ctx, 0)? == -19 {
        return Ok(false);
    }

    if ctx.i32_at(AppContext::CHAPTER_MODE)? != 3 {
        return Ok(false);
    }

    Ok(!labyrinth_active(ctx)?)
}
