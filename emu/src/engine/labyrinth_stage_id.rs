use crate::Fault;

use super::AppContext;

pub fn labyrinth_stage_id(ctx: &AppContext, stage: i32) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::LABYRINTH.wrapping_add(0x360).wrapping_add((stage as i64 as usize).wrapping_mul(4)))
}
