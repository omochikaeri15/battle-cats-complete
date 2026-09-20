use crate::Fault;

use super::{
    AppContext, AssetStream, cell_is_int, get_column_count, open_asset_stream, read_csv_cell,
    read_tsv_row,
};

pub fn load_nyancombo_param_tsv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.combo_store.params.clear();

    let Some(bytes) = open_asset_stream(ctx, b"NyancomboParam.tsv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');
    let mut row = 0i32;

    while row != 0x1d {
        read_tsv_row(stm);

        let mut values: Vec<i32> = Vec::new();
        let mut column = 0i32;

        while column < get_column_count(stm) as i32 && cell_is_int(stm, column) {
            values.push(read_csv_cell(stm, column) as i32);
            column += 1;
        }

        ctx.combo_store.params.push(values);
        row += 1;
    }

    Ok(())
}
