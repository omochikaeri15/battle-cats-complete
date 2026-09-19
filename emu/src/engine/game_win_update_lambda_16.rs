use crate::Fault;

use super::{dialog_close, mission_tracker_refresh, play_sound, sound_manager, AppContext};

pub fn game_win_update_lambda_16(ctx: &mut AppContext, dialog: u64, event: i32, button: i32) -> Result<(), Fault> {
    if event != 2 || button == -1 {
        return Ok(());
    }

    dialog_close(ctx, dialog)?;
    play_sound(sound_manager(ctx)?, 0xb, None);

    if button != 0 {
        return ctx.set_block_at::<1>(AppContext::EX_OFFERED, [0]);
    }

    ctx.set_i32_at(AppContext::RESULT_OK_PRESS, 0)?;
    ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1])?;
    ctx.set_i32_at(AppContext::CURTAIN_STYLE, 1)?;
    ctx.set_block_at::<1>(AppContext::EX_ACCEPTED, [1])?;

    mission_tracker_refresh(ctx)
}
