use crate::Fault;

use super::{play_sound, sound_manager, AppContext};

pub fn trial_win_update_lambda_1(ctx: &mut AppContext, _dialog: u64, event: i32, _button: i32) -> Result<(), Fault> {
    if event == 1 {
        play_sound(sound_manager(ctx)?, 0x1d, None);
    }

    Ok(())
}
