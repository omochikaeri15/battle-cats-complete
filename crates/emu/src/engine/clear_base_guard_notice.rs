use crate::Fault;

use super::AppContext;

pub fn clear_base_guard_notice(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::BASE_GUARD_NOTICE, 0)
}
