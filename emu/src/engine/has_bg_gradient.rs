use crate::Fault;

use super::{AppContext, BgSetup};

pub fn has_bg_gradient(ctx: &AppContext, setup: usize) -> Result<bool, Fault> {
    let top = ctx.i32_at(setup + BgSetup::GRADIENT_TOP)? as u32 >= 0x1000000;
    let bottom = ctx.i32_at(setup + BgSetup::GRADIENT_BOTTOM)? as u32 >= 0x1000000;

    Ok(bottom | top)
}
