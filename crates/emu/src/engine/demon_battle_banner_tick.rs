use crate::Fault;

use super::{AppContext, get_anim_len};

pub fn demon_battle_banner_tick(ctx: &mut AppContext) -> Result<(), Fault> {
    let frame = ctx.i32_at(AppContext::DEMON_BANNER_FRAME)?;

    if frame < 0 {
        return Ok(());
    }

    ctx.set_i32_at(AppContext::DEMON_BANNER_FRAME, frame.wrapping_add(1))?;

    if frame >= get_anim_len(&ctx.demon_banner_anim)?.wrapping_add(0x1d) {
        ctx.set_i32_at(AppContext::DEMON_BANNER_FRAME, -1)?;
    }

    Ok(())
}
