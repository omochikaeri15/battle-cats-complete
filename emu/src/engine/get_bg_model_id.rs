use crate::Fault;

use super::AppContext;

pub fn get_bg_model_id(ctx: &AppContext) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::BG_MODEL_ID)
}
