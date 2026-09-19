use crate::Fault;

use super::{get_map_type, labyrinth_active, AppContext};

pub fn is_scored_stage(ctx: &mut AppContext) -> Result<bool, Fault> {
    if ctx.u8_at(AppContext::SCORE_MODE_FLAG)? == 0 {
        return Ok(false);
    }

    if labyrinth_active(ctx)? {
        return Ok(false);
    }

    Ok(get_map_type(ctx, 0)? != -0x18)
}
