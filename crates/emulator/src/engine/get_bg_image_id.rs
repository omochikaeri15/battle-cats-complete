use crate::Fault;

use super::AppContext;

pub fn get_bg_image_id(ctx: &AppContext) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::BG_IMAGE_ID)
}
