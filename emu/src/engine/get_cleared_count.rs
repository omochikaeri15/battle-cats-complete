use crate::Fault;

use super::AppContext;

pub fn get_cleared_count(ctx: &AppContext, obj: usize) -> Result<i32, Fault> {
    ctx.i32_at(obj.wrapping_add(0x338))
}
