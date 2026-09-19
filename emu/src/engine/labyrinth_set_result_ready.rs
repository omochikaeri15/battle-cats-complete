use crate::Fault;

use super::AppContext;

pub fn labyrinth_set_result_ready(ctx: &mut AppContext, ready: u8) -> Result<(), Fault> {
    ctx.set_block_at::<1>(AppContext::LABYRINTH_RESULT_READY, [ready])
}
