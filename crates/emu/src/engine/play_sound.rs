use crate::Fault;

use super::{AppContext, play_audio};

pub fn play_sound(ctx: &mut AppContext, sound_id: i32, volume: Option<i32>) -> Result<(), Fault> {
    play_audio(ctx, sound_id, volume, false)
}
