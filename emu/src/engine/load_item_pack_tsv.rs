use crate::Fault;

use super::{
    AppContext, AssetStream, Cell, get_column_count, open_asset_stream, read_asset_stream_line,
    read_cell_stream, read_csv_cell, read_tsv_row,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ItemPackRow {
    pub server: i32,
    pub priority: i32,
    pub limit_id: i32,
    pub display: i32,
    pub product_name: Vec<u8>,
    pub html_name: Vec<u8>,
    pub items: Vec<[i32; 3]>,
    pub user_rank: i32,
    pub day: i32,
    pub limit_id2: i32,
    pub server_set: u8,
}

pub fn load_item_pack_tsv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.item_pack_rows.clear();

    let Some(bytes) = open_asset_stream(ctx, b"itemPack.tsv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');
    let mut line = Cell { at: 0, len: 0 };

    read_asset_stream_line(stm, &mut line);

    while read_tsv_row(stm) {
        if read_cell_stream(stm, 0).is_empty() {
            return Ok(());
        }

        if read_csv_cell(stm, 1) == 0 {
            continue;
        }

        let mut row = ItemPackRow {
            limit_id2: -1,
            ..ItemPackRow::default()
        };

        row.display = read_csv_cell(stm, 2) as i32;
        row.server = read_csv_cell(stm, 3) as i32;
        row.server_set = u8::from(read_csv_cell(stm, 4) as i32 == 1);
        row.limit_id = read_csv_cell(stm, 5) as i32;
        row.product_name = read_cell_stream(stm, 6).to_vec();
        row.html_name = read_cell_stream(stm, 7).to_vec();
        row.priority = read_csv_cell(stm, 8) as i32;
        row.limit_id2 = read_csv_cell(stm, 9) as i32;
        row.user_rank = read_csv_cell(stm, 0xa) as i32;
        row.day = read_csv_cell(stm, 0xb) as i32;

        let mut column = 0xc_i32;

        while column < get_column_count(stm) as i32 {
            if read_cell_stream(stm, column).is_empty() {
                break;
            }

            let category = read_csv_cell(stm, column) as i32;
            let item = read_csv_cell(stm, column.wrapping_add(1)) as i32;
            let quantity = read_csv_cell(stm, column.wrapping_add(2)) as i32;

            row.items.push([category, item, quantity]);
            column = column.wrapping_add(3);
        }

        let key = read_csv_cell(stm, 0) as i32;

        ctx.item_pack_rows.insert(key, row);
    }

    Ok(())
}
