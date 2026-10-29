use crate::Fault;

use super::{AppContext, BgSetup};

pub fn bg_has_upper_layer(ctx: &AppContext, setup: usize) -> Result<u8, Fault> {
    ctx.u8_at(setup + BgSetup::HAS_UPPER_LAYER)
}
