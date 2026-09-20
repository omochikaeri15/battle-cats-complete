use crate::Fault;

use super::{AppContext, AssetStream, cell_is_int, open_asset_stream, read_csv_cell, read_csv_row};

pub fn load_autoset_rating_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.autoset_ratings.clear();

    let Some(bytes) = open_asset_stream(ctx, b"autoset_chara_rating.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) {
        if !cell_is_int(stm, 0) {
            break;
        }

        let group = read_csv_cell(stm, 0) as i32;
        let unit = read_csv_cell(stm, 1) as i32;
        let rating = read_csv_cell(stm, 2) as i32;

        ctx.autoset_ratings
            .entry(group)
            .or_default()
            .insert(unit, rating);
    }

    Ok(())
}
