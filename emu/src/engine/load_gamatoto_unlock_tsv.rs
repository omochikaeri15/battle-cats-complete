use crate::Fault;

use super::{AppContext, AssetStream, open_asset_stream, read_csv_cell, read_tsv_row};

pub fn load_gamatoto_unlock_tsv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.gamatoto_unlocks.clear();

    let Some(bytes) = open_asset_stream(ctx, b"GamatotoUnlockData.tsv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_tsv_row(stm);

    while read_tsv_row(stm) {
        let key = read_csv_cell(stm, 0) as i32;
        let kind = read_csv_cell(stm, 1) as i32;
        let user_rank = read_csv_cell(stm, 2) as i32;
        let level = read_csv_cell(stm, 3) as i32;

        ctx.gamatoto_unlocks.insert(key, [kind, user_rank, level]);
    }

    Ok(())
}
