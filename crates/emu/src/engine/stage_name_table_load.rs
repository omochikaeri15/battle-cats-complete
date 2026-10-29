use crate::Fault;

use super::{
    AppContext, AssetStream, StageNameTable, get_column_count, open_asset_stream, read_cell_stream,
    read_stream_row, std_string_equals,
};

pub fn stage_name_table_load(
    ctx: &mut AppContext,
    table: &mut StageNameTable,
    name: &[u8],
    delimiter: u8,
) -> Result<(), Fault> {
    table.names.clear();

    if let Some(bytes) = open_asset_stream(ctx, name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_stream_row(stm, delimiter) {
            let mut idx = 0i32;

            while idx < get_column_count(stm) as i32 {
                let text = read_cell_stream(stm, idx).to_vec();

                if std_string_equals(&text, b"@") {
                    break;
                }

                if idx == 0 {
                    table.names.push(Vec::new());
                }

                let text = read_cell_stream(stm, idx).to_vec();

                if let Some(row) = table.names.last_mut() {
                    row.push(text);
                }

                idx += 1;
            }
        }
    }

    Ok(())
}
