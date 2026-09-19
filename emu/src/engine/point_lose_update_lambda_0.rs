use crate::Fault;

use super::{dialog_close, play_sound, sound_manager, AppContext};

pub fn point_lose_update_lambda_0(ctx: &mut AppContext, dialog: u64, event: i32, button: i32) -> Result<(), Fault> {
    if event != 2 || button != 0 {
        return Ok(());
    }

    play_sound(sound_manager(ctx)?, 0xb, None);

    dialog_close(ctx, dialog)
}
