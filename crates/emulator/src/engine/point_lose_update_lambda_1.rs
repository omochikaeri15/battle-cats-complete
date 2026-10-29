use crate::Fault;

use super::AppContext;

pub fn point_lose_update_lambda_1(ctx: &mut AppContext, _dialog: u64) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::OUTRO_FRAME, 0xf)
}
