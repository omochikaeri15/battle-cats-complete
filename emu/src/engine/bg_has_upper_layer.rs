use crate::Fault;

use super::AppContext;

pub fn bg_has_upper_layer(ctx: &AppContext, setup: usize) -> Result<u8, Fault> {
    ctx.u8_at(setup + 0x14)
}
