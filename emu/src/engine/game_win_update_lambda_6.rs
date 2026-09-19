use crate::Fault;

use super::{AppContext, dialog_close, play_sound, sound_manager};

pub fn game_win_update_lambda_6(
    ctx: &mut AppContext,
    dialog: u64,
    event: i32,
    _button: i32,
) -> Result<(), Fault> {
    match event {
        5 => {
            if !ctx.reward_queue.is_empty() {
                ctx.reward_queue.remove(0);
            }

            ctx.set_i32_at(AppContext::OUTRO_PHASE, 3)?;
            ctx.set_i32_at(AppContext::OUTRO_FRAME, 0x1e)
        }
        2 => {
            play_sound(sound_manager(ctx)?, 0xb, None);

            dialog_close(ctx, dialog)
        }
        _ => Ok(()),
    }
}
