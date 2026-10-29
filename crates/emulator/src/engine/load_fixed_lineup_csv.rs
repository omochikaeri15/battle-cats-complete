use crate::Fault;

use super::{
    AppContext, AssetStream, FixedLineupRow, open_asset_stream, read_cell_stream, read_csv_cell,
    read_csv_row,
};

pub fn load_fixed_lineup_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.fixed_lineup_store.rows.clear();

    let Some(bytes) = open_asset_stream(ctx, b"fixed_formation.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);

    while read_csv_row(stm) {
        ctx.fixed_lineup_store.rows.push(FixedLineupRow::default());

        let value = read_csv_cell(stm, 0) as i32;

        if let Some(row) = ctx.fixed_lineup_store.rows.last_mut() {
            row.map_id = value;
        }

        let value = read_csv_cell(stm, 1) as i32;

        if let Some(row) = ctx.fixed_lineup_store.rows.last_mut() {
            row.level = value;
        }

        let value = read_csv_cell(stm, 2) as i32;

        if let Some(row) = ctx.fixed_lineup_store.rows.last_mut() {
            row.stage = value;
        }

        let text = String::from_utf8_lossy(read_cell_stream(stm, 3)).into_owned();

        if let Some(row) = ctx.fixed_lineup_store.rows.last_mut() {
            row.name = text;
        }
    }

    Ok(())
}
