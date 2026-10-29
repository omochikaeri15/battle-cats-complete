use crate::Fault;

use super::{AppContext, AssetStream, cell_is_int, open_asset_stream, read_csv_cell, read_csv_row};

pub fn load_autoset_exclusion_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.autoset_excluded_enemies.clear();

    let Some(bytes) = open_asset_stream(ctx, b"autoset_exclude_enemy.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) {
        if !cell_is_int(stm, 0) {
            break;
        }

        let enemy = read_csv_cell(stm, 0) as i32;

        ctx.autoset_excluded_enemies.push(enemy);
    }

    Ok(())
}
