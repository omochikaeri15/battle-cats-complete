use crate::Fault;

use super::{
    AppContext, AssetStream, cell_is_int, get_column_count, open_asset_stream, read_csv_cell,
    read_csv_row,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ChangeCondition {
    pub first: i32,
    pub second: i32,
    pub keys: Vec<i32>,
    pub values: Vec<i32>,
}

pub fn load_change_conditions_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.change_conditions.clear();

    let Some(bytes) = open_asset_stream(ctx, b"change_conditions.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) {
        if !cell_is_int(stm, 0) {
            break;
        }

        let key = read_csv_cell(stm, 0) as i32;
        let value = read_csv_cell(stm, 1) as i32;

        ctx.change_conditions.entry(key).or_default().first = value;

        let value = read_csv_cell(stm, 2) as i32;

        ctx.change_conditions.entry(key).or_default().second = value;
        ctx.change_conditions.entry(key).or_default().keys.clear();
        ctx.change_conditions.entry(key).or_default().values.clear();

        let mut column = 3i32;

        while column.wrapping_add(1) < get_column_count(stm) as i32 {
            if !cell_is_int(stm, column) {
                break;
            }

            if !cell_is_int(stm, column.wrapping_add(1)) {
                break;
            }

            let value = read_csv_cell(stm, column) as i32;

            ctx.change_conditions.entry(key).or_default().keys.push(value);

            let value = read_csv_cell(stm, column.wrapping_add(1)) as i32;

            ctx.change_conditions
                .entry(key)
                .or_default()
                .values
                .push(value);

            column = column.wrapping_add(2);
        }
    }

    Ok(())
}
