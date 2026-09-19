use crate::Fault;

use super::AppContext;

pub fn scene_ignores_insets(ctx: &AppContext) -> Result<u8, Fault> {
    if ctx.i32_at(AppContext::SCENE_ID)? == 0x64 {
        let state = ctx.i32_at(AppContext::SCENE_0X64_PAGE)?;

        if state != 0 && state != 0x1869f {
            return Ok((ctx.u8_at(AppContext::INSETS_IGNORED)? != 0) as u8);
        }
    }

    Ok(1)
}
