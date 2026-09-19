use crate::Fault;

use super::{AppContext, bgm_player_pick, get_global_map_id, play_sound, sound_manager};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MapRecord {
    pub victory_voice_mode: i32,
    pub victory_voices: Vec<i32>,
    pub defeat_voice_mode: i32,
    pub defeat_voices: Vec<i32>,
    pub bgm_delay: i32,
    pub start_voice: i32,
}

pub fn bgm_player_switch(ctx: &mut AppContext, boss: i32, keep_playing: u8) -> Result<(), Fault> {
    if !ctx.bgm_player_bound {
        return Ok(());
    }

    if boss == 1 {
        sound_manager(ctx)?.stop_audio(-1);
        play_sound(sound_manager(ctx)?, 0x22, None);
        ctx.set_block_at::<1>(AppContext::BGM_PENDING, [0])?;
        ctx.set_block_at::<8>(AppContext::BGM_DELAY_FRAME, [0; 8])?;

        return Ok(());
    }

    if keep_playing == 0 {
        sound_manager(ctx)?.stop_audio(-1);

        let map_id = get_global_map_id(ctx, 0)?;
        let delay = ctx
            .map_records
            .range(map_id..)
            .next()
            .filter(|(key, _)| **key <= map_id)
            .map_or(0, |(_, record)| record.bgm_delay);

        ctx.set_block_at::<1>(AppContext::BGM_PENDING, [(delay > 0) as u8])?;
        ctx.set_i32_at(AppContext::BGM_DELAY_FRAME, 0)?;
        ctx.set_i32_at(AppContext::BGM_DELAY, if delay > 0 { delay } else { 0 })?;

        if delay > 0 {
            return Ok(());
        }
    }

    let music = bgm_player_pick(ctx)?;

    if music == -1 {
        return Ok(());
    }

    play_sound(sound_manager(ctx)?, music, None);

    Ok(())
}
