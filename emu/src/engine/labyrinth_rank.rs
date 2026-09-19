use crate::Fault;

use super::AppContext;

pub fn labyrinth_rank(ctx: &AppContext) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::LABYRINTH_RANK)
}
