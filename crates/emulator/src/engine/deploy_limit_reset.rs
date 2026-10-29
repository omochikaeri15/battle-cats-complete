use crate::Fault;

use super::AppContext;

pub fn deploy_limit_reset(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::DEPLOY_LIMIT_TOTAL, 0)
}
