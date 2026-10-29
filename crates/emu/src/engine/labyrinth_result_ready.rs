use crate::Fault;

use super::AppContext;

pub fn labyrinth_result_ready(ctx: &AppContext) -> Result<bool, Fault> {
    Ok(ctx.u8_at(AppContext::LABYRINTH_RESULT_READY)? != 0)
}
