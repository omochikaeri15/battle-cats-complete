use crate::Fault;

use super::{AppContext, AssetStream, open_asset_stream, read_csv_cell, read_csv_row};

pub fn load_gold_cpu_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.gold_cpu_rows.clear();
    ctx.lock_skip_rows.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"ScatCPUsetting.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            let first = read_csv_cell(stm, 0) as i32;
            let second = read_csv_cell(stm, 1) as i32;
            let third = read_csv_cell(stm, 2) as i32;
            let fourth = read_csv_cell(stm, 3) as i32;

            ctx.gold_cpu_rows.push([first, second, third, fourth]);
        }
    }

    let Some(bytes) = open_asset_stream(ctx, b"LockSkipData.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) {
        let first = read_csv_cell(stm, 0) as i32;
        let second = read_csv_cell(stm, 1) as i32;

        ctx.lock_skip_rows.push([first, second]);
    }

    Ok(())
}
