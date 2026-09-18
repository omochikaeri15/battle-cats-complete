use crate::Fault;

use super::{get_scene_id, invasion_available, AppContext};

pub fn is_invasion_stage(ctx: &AppContext) -> Result<bool, Fault> {
    if get_scene_id(ctx)? == 0x12c {
        return Ok(ctx.u8_at(0x32c5d6)? != 0);
    }

    if !invasion_available(ctx, ctx.i32_at(0x327efc)?)? {
        return Ok(false);
    }

    Ok(ctx.i32_at(0x325c48)? == ctx.i8_at(0x32c5d8)? as i32)
}
