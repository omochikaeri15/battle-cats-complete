use crate::Fault;

use super::AppContext;

pub fn get_castle_id(ctx: &AppContext) -> Result<i32, Fault> {
    if ctx.i32_at(AppContext::CHAPTER_MODE)? == 3 {
        return ctx.i32_at(AppContext::STAGE_CASTLE_ID);
    }

    if ctx.i32_at(AppContext::CHAPTER_MODE)? == 0x63 {
        return ctx.i32_at(AppContext::STAGE_CASTLE_ID);
    }

    ctx.i32_at(AppContext::CASTLE_ID)
}
