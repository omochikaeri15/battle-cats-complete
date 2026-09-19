use crate::Fault;

use super::AppContext;

pub fn get_bg_gradient_bottom(ctx: &AppContext, setup: usize) -> Result<i32, Fault> {
    ctx.i32_at(setup + 0x20)
}
