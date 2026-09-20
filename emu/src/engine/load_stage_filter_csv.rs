use crate::Fault;

use super::{AppContext, AssetStream, cell_is_int, open_asset_stream, read_csv_cell, read_csv_row};

pub fn load_stage_filter_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.filter_stage_maps.clear();

    let Some(bytes) = open_asset_stream(ctx, b"FilterStage_Setting.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);

    while read_csv_row(stm) {
        if !cell_is_int(stm, 0) {
            break;
        }

        ctx.filter_stage_maps.push(read_csv_cell(stm, 0) as i32);
    }

    Ok(())
}
