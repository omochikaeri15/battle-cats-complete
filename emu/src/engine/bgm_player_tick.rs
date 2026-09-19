use crate::Fault;

use super::{AppContext, bgm_player_pick, play_sound, sound_manager};

pub fn bgm_player_tick(ctx: &mut AppContext) -> Result<(), Fault> {
    if !ctx.bgm_player_bound {
        return Ok(());
    }

    if ctx.u8_at(AppContext::BGM_PENDING)? == 0 {
        return Ok(());
    }

    let frame = ctx.i32_at(AppContext::BGM_DELAY_FRAME)?.wrapping_add(1);

    ctx.set_i32_at(AppContext::BGM_DELAY_FRAME, frame)?;

    if frame != ctx.i32_at(AppContext::BGM_DELAY)? {
        return Ok(());
    }

    let music = bgm_player_pick(ctx)?;

    if music == -1 {
        return Ok(());
    }

    play_sound(sound_manager(ctx)?, music, None);

    Ok(())
}
