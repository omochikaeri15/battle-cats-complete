use crate::Fault;

use super::{get_scene_id, invasion_z_available, AppContext};

pub fn is_z_invasion_stage(ctx: &mut AppContext) -> Result<bool, Fault> {
    if get_scene_id(ctx)? == 0x12c {
        return Ok(ctx.u8_at(0x32c5d7)? != 0);
    }

    let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;

    if !invasion_z_available(ctx, chapter)? {
        return Ok(false);
    }

    Ok(ctx.i32_at(AppContext::STAGE_INDEX)? == ctx.i8_at(0x32c5d8)? as i32)
}
