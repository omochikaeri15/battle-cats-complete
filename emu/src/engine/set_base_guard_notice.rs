use crate::Fault;

use super::AppContext;

pub fn set_base_guard_notice(ctx: &mut AppContext, notice: usize, state: i32) -> Result<(), Fault> {
    if state == 1 && ctx.i32_at(notice)? == 1 {
        return Ok(());
    }

    ctx.set_i32_at(notice, state)?;
    ctx.set_i32_at(notice.wrapping_add(4), 0)
}
