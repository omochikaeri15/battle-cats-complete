use crate::Fault;

use super::AppContext;

pub fn set_auto_camera_mode(ctx: &mut AppContext, mode: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::AUTO_CAMERA_MODE, mode)
}
