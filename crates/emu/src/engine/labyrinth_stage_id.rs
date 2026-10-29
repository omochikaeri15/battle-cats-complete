use crate::Fault;

use super::AppContext;

pub fn labyrinth_stage_id(ctx: &AppContext, stage: i32) -> Result<i32, Fault> {
    ctx.i32_at(
        AppContext::LABYRINTH_STAGE_IDS.wrapping_add((stage as i64 as usize).wrapping_mul(4)),
    )
}
