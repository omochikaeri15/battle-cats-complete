use crate::Fault;

use super::{AppContext, BgSetup};

pub fn get_bg_ground_top(ctx: &AppContext, setup: usize) -> Result<i32, Fault> {
    ctx.i32_at(setup + BgSetup::GROUND_TOP)
}
