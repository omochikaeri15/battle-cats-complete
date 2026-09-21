use crate::Fault;

use super::{
    AppContext, AssetStream, Cell, get_column_count, open_asset_stream, read_asset_stream_line,
    read_csv_cell, read_csv_row,
};

pub fn load_cat_drops_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.drop_chara_rows.clear();
    ctx.drop_chara_max_1100 = -1;
    ctx.drop_chara_max_1000 = -1;

    let Some(bytes) = open_asset_stream(ctx, b"drop_chara.csv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');
    let mut line = Cell { at: 0, len: 0 };

    read_asset_stream_line(stm, &mut line);

    while read_csv_row(stm) {
        if (get_column_count(stm) as i32) < 3 {
            continue;
        }

        let row = vec![
            read_csv_cell(stm, 0) as i32,
            read_csv_cell(stm, 1) as i32,
            read_csv_cell(stm, 2) as i32,
        ];
        let id = row.first().copied().unwrap_or(0);

        if (id.wrapping_sub(0x3e8) as u32) < 0x64 {
            if id > ctx.drop_chara_max_1000 {
                ctx.drop_chara_max_1000 = id;
            }
        } else if (id.wrapping_sub(0x44c) as u32) <= 0x63 && id > ctx.drop_chara_max_1100 {
            ctx.drop_chara_max_1100 = id;
        }

        ctx.drop_chara_rows.push(row);
    }

    Ok(())
}
