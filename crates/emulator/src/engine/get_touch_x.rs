use crate::Fault;

use super::AppContext;

pub fn get_touch_x(ctx: &AppContext) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::TOUCH_X)
}
