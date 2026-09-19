use crate::Fault;

use super::AppContext;

pub fn get_score_time_limit(ctx: &AppContext) -> Result<i32, Fault> {
    Ok(ctx.i32_at(AppContext::STAGE_SCORE_TIME_LIMIT)?.wrapping_mul(0x708))
}
