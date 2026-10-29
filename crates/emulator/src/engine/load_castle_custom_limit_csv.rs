use crate::Fault;

use super::{AppContext, AssetStream, open_asset_stream, read_csv_cell, read_csv_row};

pub fn load_castle_custom_limit_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    let Some(bytes) = open_asset_stream(ctx, b"CastleCustomLimit.csv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);
    ctx.castle_custom_limit = read_csv_cell(stm, 0) as i32;

    Ok(())
}
