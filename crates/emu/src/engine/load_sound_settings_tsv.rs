use crate::Fault;

use super::{
    AppContext, AssetStream, Cell, get_column_count, open_asset_stream, read_asset_stream_line,
    read_csv_cell, read_tsv_row,
};

pub fn load_sound_settings_tsv(ctx: &mut AppContext, name: &[u8]) -> Result<(), Fault> {
    ctx.sound_manager.kinds.clear();
    ctx.sound_manager.priorities.clear();

    if let Some(bytes) = open_asset_stream(ctx, name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut line = Cell { at: 0, len: 0 };

        read_asset_stream_line(stm, &mut line);

        while read_tsv_row(stm) && get_column_count(stm) as i64 >= 3 {
            let bgm = i32::from(read_csv_cell(stm, 0) as i32 == 1);
            let looping = i32::from(read_csv_cell(stm, 1) as i32 == 1);

            ctx.sound_manager
                .kinds
                .push(looping.wrapping_mul(4).wrapping_sub(bgm).wrapping_add(2));

            let priority = read_csv_cell(stm, 2) as i32;

            ctx.sound_manager.priorities.push(priority);
        }
    }

    let mgr = &mut ctx.sound_manager;

    mgr.count = mgr.kinds.len() as u64 as i32;

    let count = mgr.count as i64 as usize;

    mgr.sound_data.truncate(count);
    mgr.sound_data.resize_with(count, || None);
    mgr.sources.truncate(count);
    mgr.sources.resize_with(count, || None);
    mgr.looping.resize(count, false);
    mgr.volumes.resize(count, 0);

    for slot in mgr.looping.iter_mut() {
        *slot = false;
    }

    for slot in mgr.volumes.iter_mut() {
        *slot = 0x64;
    }

    Ok(())
}
