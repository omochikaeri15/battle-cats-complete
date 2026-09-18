use crate::fault::Fault;

use super::AppContext;

pub fn read_flag(ctx: &AppContext, flag: usize) -> Result<i32, Fault> {
    ctx.i32_at(flag)
}
