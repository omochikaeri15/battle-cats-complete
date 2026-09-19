use crate::Fault;

use super::AppContext;

pub fn touch_is_down(ctx: &AppContext) -> Result<u8, Fault> {
    ctx.u8_at(AppContext::TOUCH_IS_DOWN)
}
