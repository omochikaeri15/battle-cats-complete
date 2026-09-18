use crate::Fault;

use super::{get_scene_id, invasion_available, AppContext};

pub fn is_invasion_stage(ctx: &AppContext) -> Result<bool, Fault> {
    if get_scene_id(ctx)? == 0x12c {
        return Ok(ctx.u8_at(AppContext::BATTLE_IS_INVASION)? != 0);
    }

    if !invasion_available(ctx, ctx.i32_at(AppContext::CHAPTER_MODE)?)? {
        return Ok(false);
    }

    Ok(ctx.i32_at(AppContext::STAGE_INDEX)? == ctx.i8_at(AppContext::INVASION_STAGE)? as i32)
}
