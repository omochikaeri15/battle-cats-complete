use crate::Fault;

use super::{
    AppContext, fs_asset_root, fs_file_exists, fs_files_dir, fs_remove_file, path_join, snd_load_sound_data, sound_manager,
    string_format_int,
};

pub fn sound_load(ctx: &mut AppContext, sound_id: i32) -> Result<(), Fault> {
    let root = fs_files_dir(ctx)?;
    let name = string_format_int(ctx, b"snd%03d.ogg", sound_id)?;
    let mut path = path_join(&root, &name);

    if fs_file_exists(ctx, &path)? {
        fs_remove_file(ctx, &path)?;
    }

    let index = sound_id as i64 as usize;
    let mgr = sound_manager(ctx)?;
    let limit = mgr.kinds.len() as i64;

    if *mgr.kinds.get(index).ok_or(Fault::index_out_of_range(index as i64, limit))? & 2 == 0 {
        return Ok(());
    }

    let limit = mgr.sound_data.len() as i64;

    if mgr.sound_data.get(index).ok_or(Fault::index_out_of_range(index as i64, limit))?.is_some() {
        return Ok(());
    }

    let root = fs_asset_root();
    let name = string_format_int(ctx, b"snd%03d.caf", sound_id)?;

    path = path_join(&root, &name);

    if !fs_file_exists(ctx, &path)? {
        if sound_manager(ctx)?.caf_lookup_mode != 1 {
            let root = fs_files_dir(ctx)?;
            let name = string_format_int(ctx, b"%03d.caf", sound_id)?;

            path = path_join(&root, &name);
        } else {
            let root = fs_files_dir(ctx)?;
            let name = string_format_int(ctx, b"%03d.caf", sound_id)?;

            path = path_join(&root, &name);
        }
    }

    let data = snd_load_sound_data(ctx, &path)?;

    sound_manager(ctx)?.sound_data[index] = data;

    Ok(())
}
