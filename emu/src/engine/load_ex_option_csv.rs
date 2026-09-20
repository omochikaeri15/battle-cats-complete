use crate::Fault;

use super::{
    AppContext, AssetStream, cell_is_int, get_column_count, open_asset_stream, read_csv_cell,
    read_csv_row,
};

pub fn load_ex_option_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.ex_option_targets.clear();

    let Some(bytes) = open_asset_stream(ctx, b"EX_option.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) {
        if (get_column_count(stm) as i32) < 2 {
            break;
        }

        if !cell_is_int(stm, 0) {
            break;
        }

        let value = read_csv_cell(stm, 1) as i32;
        let key = read_csv_cell(stm, 0) as i32;

        ctx.ex_option_targets.insert(key, value);
    }

    Ok(())
}
