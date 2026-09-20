use crate::Fault;

use super::{get_column_count, open_asset_stream, read_cell_stream, read_tsv_row, AppContext, AssetStream};

pub fn load_parameter_table_tsv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.settings.clear();

    let Some(bytes) = open_asset_stream(ctx, b"param.tsv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_tsv_row(stm) {
        if (get_column_count(stm) as i64) < 2 {
            continue;
        }

        let value = read_cell_stream(stm, 1).to_vec();
        let key = read_cell_stream(stm, 0).to_vec();

        ctx.settings.insert(key, value);
    }

    Ok(())
}
