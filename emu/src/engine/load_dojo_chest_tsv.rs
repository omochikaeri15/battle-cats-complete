use crate::Fault;

use super::{AppContext, AssetStream, open_asset_stream, parse_dojo_chest_row, read_tsv_row};

pub fn load_dojo_chest_tsv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.dojo_chest_rows.clear();

    let Some(bytes) = open_asset_stream(ctx, b"TreasureBox.tsv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_tsv_row(stm);

    while read_tsv_row(stm) {
        let row = parse_dojo_chest_row(stm);

        ctx.dojo_chest_rows.push(row);
    }

    Ok(())
}
