use crate::Fault;

use super::{get_scene_id, AppContext};

pub fn is_outbreak_stage(ctx: &AppContext) -> Result<bool, Fault> {
    if get_scene_id(ctx)? == 0x12c {
        return Ok(ctx.u8_at(0x32c5c8)? != 0);
    }

    if ctx.u8_at(0x32c5c9)? == 0 {
        return Ok(false);
    }

    if !ctx.outbreak_active.contains_key(&ctx.i32_at(AppContext::CHAPTER_MODE)?) {
        return Ok(false);
    }

    let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
    let stages =
        ctx.outbreak_active.get(&chapter).ok_or(Fault::KeyNotFound { site: "is_outbreak_stage", key: chapter as i64 })?;

    if !stages.contains_key(&ctx.i32_at(AppContext::STAGE_INDEX)?) {
        return Ok(false);
    }

    let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
    let stages =
        ctx.outbreak_active.get(&chapter).ok_or(Fault::KeyNotFound { site: "is_outbreak_stage", key: chapter as i64 })?;
    let stage = ctx.i32_at(AppContext::STAGE_INDEX)?;

    stages.get(&stage).copied().ok_or(Fault::KeyNotFound { site: "is_outbreak_stage", key: stage as i64 })
}
