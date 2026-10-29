use crate::Fault;

use super::AppContext;

pub fn back_pressed(ctx: &AppContext) -> Result<u8, Fault> {
    ctx.u8_at(AppContext::BACK_PRESSED)
}
