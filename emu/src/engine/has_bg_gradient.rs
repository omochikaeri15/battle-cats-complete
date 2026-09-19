use crate::Fault;

use super::AppContext;

pub fn has_bg_gradient(ctx: &AppContext, setup: usize) -> Result<bool, Fault> {
    let top = ctx.i32_at(setup + 0x1c)? as u32 >= 0x1000000;
    let bottom = ctx.i32_at(setup + 0x20)? as u32 >= 0x1000000;

    Ok(bottom | top)
}
