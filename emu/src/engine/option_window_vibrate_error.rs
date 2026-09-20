use crate::Fault;

use super::{AppContext, dialog_close, play_sound, sound_manager};

pub fn option_window_vibrate_error(
    ctx: &mut AppContext,
    dialog: u64,
    event: i32,
    button: i32,
) -> Result<(), Fault> {
    if event != 2 {
        return Ok(());
    }

    if button != 0 {
        return Ok(());
    }

    play_sound(sound_manager(ctx)?, 0xb, None);

    dialog_close(ctx, dialog)
}
