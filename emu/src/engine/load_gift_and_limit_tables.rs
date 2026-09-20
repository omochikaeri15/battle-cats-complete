use crate::Fault;

use super::{AppContext, AssetStream, get_column_count, open_asset_stream, read_csv_cell, read_csv_row};

pub fn load_gift_and_limit_tables(ctx: &mut AppContext) -> Result<(), Fault> {
    let gift = open_asset_stream(ctx, b"rankGift.csv", 0, 0)?;

    ctx.rank_gift_rows = vec![[-1i32; 21]; 400];

    if let Some(bytes) = gift {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut row = 0usize;

        while row != 400 {
            if !read_csv_row(stm) {
                break;
            }

            if get_column_count(stm) as i32 == 1 {
                break;
            }

            let mut column = 0usize;

            loop {
                let value = read_csv_cell(stm, column as i32) as i32;

                if let Some(slot) = ctx.rank_gift_rows.get_mut(row).and_then(|r| r.get_mut(column)) {
                    *slot = value;
                }

                if value == -1 || column >= 0x14 {
                    break;
                }

                column += 1;
            }

            row += 1;
        }
    }

    let limit = open_asset_stream(ctx, b"unitlimit.csv", 0, 0)?;

    ctx.unit_limit_rows = vec![[-1i32; 10]; 876];

    if let Some(bytes) = limit {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut row = 0usize;

        while row != 876 {
            if read_csv_row(stm) {
                get_column_count(stm);
            }

            let mut column = 0usize;

            loop {
                let value = read_csv_cell(stm, column as i32) as i32;

                if let Some(slot) = ctx.unit_limit_rows.get_mut(row).and_then(|r| r.get_mut(column))
                {
                    *slot = value;
                }

                if value == -1 || column >= 9 {
                    break;
                }

                column += 1;
            }

            row += 1;
        }
    }

    ctx.unit_limit_extra = [-1i32; 110];

    Ok(())
}
