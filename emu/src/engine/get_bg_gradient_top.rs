use crate::Fault;

use super::{AppContext, BgSetup};

pub fn get_bg_gradient_top(ctx: &AppContext, setup: usize) -> Result<i32, Fault> {
    ctx.i32_at(setup + BgSetup::GRADIENT_TOP)
}
