use crate::Fault;

use super::{AppContext, dialog_close, play_sound, sound_manager};

pub fn game_update_lambda_3(
    ctx: &mut AppContext,
    dialog: u64,
    event: i32,
    button: i32,
) -> Result<(), Fault> {
    if event == 2 && button == 0 {
        play_sound(sound_manager(ctx)?, 0xb, None);

        return dialog_close(ctx, dialog);
    }

    if event == 5 {
        ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [0])?;
    }

    Ok(())
}
