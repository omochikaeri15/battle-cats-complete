use crate::Fault;

use super::AppContext;

pub fn labyrinth_active(ctx: &AppContext) -> Result<bool, Fault> {
    let score_mode = ctx.u8_at(AppContext::SCORE_MODE_FLAG)? != 0;
    let active = ctx.u8_at(AppContext::LABYRINTH_ACTIVE)? != 0;

    Ok(active & score_mode)
}
