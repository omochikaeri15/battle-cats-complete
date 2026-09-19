use crate::Fault;

use super::AppContext;

pub fn get_auto_camera_mode(ctx: &AppContext) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::AUTO_CAMERA_MODE)
}
