use crate::Fault;

use super::{dialog_close, play_sound, sound_manager, AppContext};

pub fn game_win_update_lambda_12(ctx: &mut AppContext, dialog: u64, event: i32, button: i32) -> Result<(), Fault> {
    if event == 2 && button == 0 {
        play_sound(sound_manager(ctx)?, 0xb, None);

        return dialog_close(ctx, dialog);
    }

    if event != 5 {
        return Ok(());
    }

    if !ctx.reward_queue.is_empty() {
        ctx.reward_queue.remove(0);
    }

    ctx.set_i32_at(AppContext::RESULT_PHASE, 3)?;
    ctx.set_i32_at(AppContext::RESULT_FRAME, 0x1e)
}
