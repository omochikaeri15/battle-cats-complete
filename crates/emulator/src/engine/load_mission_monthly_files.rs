use crate::Fault;

use super::{
    AppContext, AssetStream, MissionMonthly, get_column_count, open_asset_stream,
    parse_mission_monthly_row, read_csv_cell, read_csv_row,
};

pub fn load_mission_monthly_files(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.mission_monthly.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"Mission_Monthly.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_csv_row(stm);

        while read_csv_row(stm) {
            let mut row = MissionMonthly::default();

            parse_mission_monthly_row(&mut row, stm);

            let key = read_csv_cell(stm, 0) as i32;

            ctx.mission_monthly.insert(key, row);
        }
    }

    ctx.mission_groups.clear();

    let Some(bytes) = open_asset_stream(ctx, b"Mission_Group.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);

    while read_csv_row(stm) {
        let mut values: Vec<i32> = Vec::new();
        let mut column = 1i32;

        while column < get_column_count(stm) as i32 {
            values.push(read_csv_cell(stm, column) as i32);
            column += 1;
        }

        let key = read_csv_cell(stm, 0) as i32;

        ctx.mission_groups.insert(key, values);
    }

    Ok(())
}
