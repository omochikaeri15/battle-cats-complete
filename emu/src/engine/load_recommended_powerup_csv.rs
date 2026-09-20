use crate::Fault;

use super::{AppContext, AssetStream, cell_is_int, open_asset_stream, read_csv_cell, read_csv_row};

pub fn load_recommended_powerup_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.recommended_powerup_rows.clear();

    let Some(bytes) = open_asset_stream(ctx, b"beginner_recommend_powerup.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) {
        if !cell_is_int(stm, 0) {
            break;
        }

        let first = read_csv_cell(stm, 0) as i32;
        let second = read_csv_cell(stm, 1) as i32;
        let third = read_csv_cell(stm, 2) as i32;
        let fourth = read_csv_cell(stm, 3) as i32;

        ctx.recommended_powerup_rows
            .push([first, second, third, fourth]);
    }

    Ok(())
}
