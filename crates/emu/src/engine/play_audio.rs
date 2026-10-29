use std::rc::Rc;

use crate::Fault;

use super::{
    AppContext, au_new_buffer_source, au_new_stream_source, fs_asset_root, fs_file_exists, fs_files_dir, path_join,
    sound_manager, sound_prepare, stop_audio, string_format_rank_comment,
};

pub fn play_audio(ctx: &mut AppContext, sound_id: i32, volume: Option<i32>, is_bgm: bool) -> Result<(), Fault> {
    if sound_manager(ctx)?.kinds.len() as u64 <= sound_id as i64 as u64 {
        return Ok(());
    }

    let mut sound = sound_id;

    if sound_id == 0x1d {
        sound = 0x1d;

        if ctx.i32_at(AppContext::EVENT_DISPLAY_ID)? != -1 {
            let event = ctx.i32_at(AppContext::EVENT_DISPLAY_ID)?;

            if ctx.event_display.contains_key(&event) {
                let event = ctx.i32_at(AppContext::EVENT_DISPLAY_ID)?;

                if ctx.event_display.entry(event).or_default().popup_get_se != -1 {
                    let event = ctx.i32_at(AppContext::EVENT_DISPLAY_ID)?;

                    sound = ctx.event_display.entry(event).or_default().popup_get_se;
                }
            }
        }
    }

    let index = sound as i64 as usize;
    let mgr = sound_manager(ctx)?;
    let limit = mgr.kinds.len() as i64;
    let kind = *mgr.kinds.get(index).ok_or(Fault::index_out_of_range(index as i64, limit))?;

    if kind & 1 != 0 && mgr.bgm_muted == 0 {
        let root = fs_asset_root();
        let name = string_format_rank_comment(ctx, b"snd%03d.%s", sound, b"ogg")?;
        let mut path = path_join(&root, &name);

        if !fs_file_exists(ctx, &path)? {
            let root = fs_files_dir(ctx)?;
            let name = string_format_rank_comment(ctx, b"%03d.%s", sound, b"ogg")?;

            path = path_join(&root, &name);
        }

        if sound_manager(ctx)?.bgm_source.is_some() {
            stop_audio(sound_manager(ctx)?, -1)?;
        }

        let mgr = sound_manager(ctx)?;
        let limit = mgr.priorities.len() as i64;
        let priority = *mgr.priorities.get(index).ok_or(Fault::index_out_of_range(index as i64, limit))?;
        let prepared = sound_prepare(ctx, priority)?;
        let mut played = false;

        if prepared {
            let source = au_new_stream_source(ctx, &path, 1)?;
            let mgr = sound_manager(ctx)?;
            let limit = mgr.sources.len() as i64;

            *mgr.sources.get_mut(index).ok_or(Fault::index_out_of_range(index as i64, limit))? = source;

            if let Some(source) = mgr.sources.get(index).cloned().flatten() {
                if mgr.kinds.get(index).is_some_and(|kind| kind & 4 != 0) {
                    source.set_loop(true);
                }

                mgr.bgm_source = Some(Rc::clone(&source));

                let level = if is_bgm {
                    mgr.bgm_duck
                } else {
                    mgr.bgm_duck = volume.unwrap_or(0x64);
                    mgr.bgm_duck
                };

                source.set_volume(level.wrapping_mul(mgr.bgm_volume) as f32 / 10000.0f32);
                source.play();
                played = true;
            }
        }

        if !played {
            return Ok(());
        }
    } else if kind & 2 != 0 && mgr.se_muted == 0 {
        let limit = mgr.sound_data.len() as i64;

        if mgr.sound_data.get(index).ok_or(Fault::index_out_of_range(index as i64, limit))?.is_none() {
            return Ok(());
        }

        if !is_bgm {
            let limit = mgr.volumes.len() as i64;

            *mgr.volumes.get_mut(index).ok_or(Fault::index_out_of_range(index as i64, limit))? = volume.unwrap_or(0x64);
        }

        let limit = mgr.sources.len() as i64;

        match mgr.sources.get(index).ok_or(Fault::index_out_of_range(index as i64, limit))?.clone() {
            Some(source) => {
                if source.is_playing() {
                    source.play();
                }
            }
            None => {
                let limit = mgr.priorities.len() as i64;
                let priority = *mgr.priorities.get(index).ok_or(Fault::index_out_of_range(index as i64, limit))?;

                if !sound_prepare(ctx, priority)? {
                    return Ok(());
                }

                let data = sound_manager(ctx)?.sound_data.get(index).cloned().flatten().ok_or(Fault::null_pointer())?;
                let source = au_new_buffer_source(ctx, &data)?;
                let mgr = sound_manager(ctx)?;

                if let Some(slot) = mgr.sources.get_mut(index) {
                    *slot = source;
                }

                if mgr.sources.get(index).is_none_or(Option::is_none) {
                    return Ok(());
                }
            }
        }

        let mgr = sound_manager(ctx)?;
        let source = mgr.sources.get(index).cloned().flatten().ok_or(Fault::null_pointer())?;

        if mgr.kinds.get(index).is_some_and(|kind| kind & 4 != 0) {
            source.set_loop(true);
        }

        let limit = mgr.volumes.len() as i64;
        let level = *mgr.volumes.get(index).ok_or(Fault::index_out_of_range(index as i64, limit))?;

        source.set_volume(level.wrapping_mul(mgr.se_volume) as f32 / 10000.0f32);
        source.play();
    }

    let mgr = sound_manager(ctx)?;

    if mgr.kinds.get(index).is_some_and(|kind| kind & 4 != 0) {
        let limit = mgr.looping.len() as i64;

        *mgr.looping.get_mut(index).ok_or(Fault::index_out_of_range(index as i64, limit))? = true;
    }

    Ok(())
}
