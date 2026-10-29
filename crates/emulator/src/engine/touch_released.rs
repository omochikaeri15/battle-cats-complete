use crate::Fault;

use super::AppContext;

pub fn touch_released(ctx: &AppContext) -> Result<u8, Fault> {
    ctx.u8_at(AppContext::TOUCH_RELEASED)
}
