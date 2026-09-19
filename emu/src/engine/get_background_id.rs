use crate::Fault;

use super::AppContext;

pub fn get_background_id(ctx: &AppContext) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::STAGE_BACKGROUND_ID)
}
