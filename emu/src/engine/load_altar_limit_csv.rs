use crate::Fault;

use super::{
    AppContext, AssetStream, get_column_count, open_asset_stream, read_cell_stream,
    read_csv_cell, read_csv_row,
};

pub fn load_altar_limit_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.altar_rewards.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"DemonCastlelimit.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            if (get_column_count(stm) as i32) < 4 {
                break;
            }

            let value = read_csv_cell(stm, 1) as i32;
            let key = read_csv_cell(stm, 0) as i32;

            ctx.altar_rewards.entry(key).or_default().amount = value;

            let value = read_csv_cell(stm, 2) as i32;
            let key = read_csv_cell(stm, 0) as i32;

            ctx.altar_rewards.entry(key).or_default().unseal = value;

            let value = read_csv_cell(stm, 3) as i32;
            let key = read_csv_cell(stm, 0) as i32;

            ctx.altar_rewards.entry(key).or_default().enemy = value;
        }
    }

    ctx.altar_enemy_ids.clear();

    let Some(bytes) = open_asset_stream(ctx, b"DemonCastledefine.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);

    while read_csv_row(stm) {
        if (get_column_count(stm) as i32) < 2 {
            break;
        }

        if read_cell_stream(stm, 0).is_empty() {
            break;
        }

        let value = read_csv_cell(stm, 1) as i32;
        let key = read_csv_cell(stm, 0) as i32;

        ctx.altar_enemy_ids.insert(key, value);
    }

    Ok(())
}
