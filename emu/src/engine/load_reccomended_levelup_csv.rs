use crate::Fault;

use super::{AppContext, AssetStream, cell_is_int, open_asset_stream, read_csv_cell, read_csv_row};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct RecommendedLevelup {
    pub values: [i32; 3],
    pub flag: u8,
    pub tail: [i32; 2],
}

pub fn load_reccomended_levelup_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.recommended_levelups.clear();

    let Some(bytes) = open_asset_stream(ctx, b"Recommended_levelup.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) {
        if !cell_is_int(stm, 0) {
            break;
        }

        let key = read_csv_cell(stm, 0) as i32;
        let value = read_csv_cell(stm, 1) as i32;

        ctx.recommended_levelups.entry(key).or_default().values[0] = value;

        let value = read_csv_cell(stm, 2) as i32;

        ctx.recommended_levelups.entry(key).or_default().values[1] = value;

        let value = read_csv_cell(stm, 3) as i32;

        ctx.recommended_levelups.entry(key).or_default().values[2] = value;

        let value = read_csv_cell(stm, 4) as i32;

        ctx.recommended_levelups.entry(key).or_default().flag = u8::from(value != 0);

        let value = read_csv_cell(stm, 5) as i32;

        ctx.recommended_levelups.entry(key).or_default().tail[0] = value;

        let value = read_csv_cell(stm, 6) as i32;

        ctx.recommended_levelups.entry(key).or_default().tail[1] = value;
    }

    Ok(())
}
