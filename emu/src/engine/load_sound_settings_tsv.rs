use crate::Fault;

use super::{
    AppContext, AssetStream, Cell, get_column_count, open_asset_stream, read_asset_stream_line,
    read_csv_cell, read_tsv_row,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct SoundState {
    pub count: i32,
    pub kinds: Vec<i32>,
    pub priorities: Vec<i32>,
    pub playing: Vec<bool>,
    pub volumes: Vec<i32>,
}

pub fn load_sound_settings_tsv(ctx: &mut AppContext, name: &[u8]) -> Result<(), Fault> {
    ctx.sound_state.kinds.clear();
    ctx.sound_state.priorities.clear();

    if let Some(bytes) = open_asset_stream(ctx, name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut line = Cell { at: 0, len: 0 };

        read_asset_stream_line(stm, &mut line);

        while read_tsv_row(stm) && get_column_count(stm) as i64 >= 3 {
            let bgm = i32::from(read_csv_cell(stm, 0) as i32 == 1);
            let looping = i32::from(read_csv_cell(stm, 1) as i32 == 1);

            ctx.sound_state
                .kinds
                .push(looping.wrapping_mul(4).wrapping_sub(bgm).wrapping_add(2));

            let priority = read_csv_cell(stm, 2) as i32;

            ctx.sound_state.priorities.push(priority);
        }
    }

    ctx.sound_state.count = ctx.sound_state.kinds.len() as u64 as i32;

    let count = ctx.sound_state.count as i64 as usize;

    ctx.sound_state.playing.resize(count, false);

    for slot in ctx.sound_state.playing.iter_mut() {
        *slot = false;
    }

    ctx.sound_state.volumes.resize(count, 0);

    for slot in ctx.sound_state.volumes.iter_mut() {
        *slot = 0x64;
    }

    Ok(())
}
