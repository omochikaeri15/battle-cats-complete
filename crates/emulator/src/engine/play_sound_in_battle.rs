use crate::Fault;

use super::{AppContext, get_battle_status, play_audio};

pub fn play_sound_in_battle(ctx: &mut AppContext, sound_id: i32) -> Result<(), Fault> {
    if get_battle_status(ctx)? != 0 {
        return Ok(());
    }

    play_audio(ctx, sound_id, None, false)
}
