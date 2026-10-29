use crate::Fault;

use super::AppContext;

pub fn get_scene_id(ctx: &AppContext) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::SCENE_ID)
}
