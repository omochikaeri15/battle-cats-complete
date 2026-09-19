use crate::Fault;

use super::{get_battle_status, sound_manager, AppContext};

pub fn play_sound_in_battle(ctx: &mut AppContext, sound_id: i32) -> Result<(), Fault> {
    if get_battle_status(ctx)? != 0 {
        return Ok(());
    }

    sound_manager(ctx)?.play_audio(sound_id, None, false);

    Ok(())
}
