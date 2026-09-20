use crate::Fault;

use super::{
    AppContext, AssetStream, ExGroup, cell_is_int, get_column_count, open_asset_stream,
    read_csv_cell, read_csv_row,
};

pub fn load_continue_stages_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.ex_groups.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"EX_group.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            if !cell_is_int(stm, 0) {
                break;
            }

            let map = read_csv_cell(stm, 1) as i32;
            let stage = read_csv_cell(stm, 2) as i32;
            let key = stage.wrapping_add(map.wrapping_mul(100));

            ctx.ex_groups.insert(key, ExGroup::default());

            let value = read_csv_cell(stm, 0) as i32;

            if let Some(group) = ctx.ex_groups.get_mut(&key) {
                group.group = value;
            }

            let mut column = 3i32;

            while column.wrapping_add(1) < get_column_count(stm) as i32 {
                if !cell_is_int(stm, column) {
                    break;
                }

                let value = read_csv_cell(stm, column) as i32;

                if let Some(group) = ctx.ex_groups.get_mut(&key) {
                    group.indices.push(value);
                }

                let value = read_csv_cell(stm, column.wrapping_add(1)) as i32;

                if let Some(group) = ctx.ex_groups.get_mut(&key) {
                    group.thresholds.push(value);
                }

                column = column.wrapping_add(2);
            }
        }
    }

    ctx.ex_lottery.clear();

    let Some(bytes) = open_asset_stream(ctx, b"EX_lottery.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) {
        if !cell_is_int(stm, 0) {
            break;
        }

        ctx.ex_lottery.push([0; 2]);

        let value = read_csv_cell(stm, 0) as i32;

        if let Some(pair) = ctx.ex_lottery.last_mut() {
            pair[0] = value;
        }

        let value = read_csv_cell(stm, 1) as i32;

        if let Some(pair) = ctx.ex_lottery.last_mut() {
            pair[1] = value;
        }
    }

    Ok(())
}
