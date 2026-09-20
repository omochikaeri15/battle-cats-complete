use crate::Fault;

use super::{
    AppContext, AssetStream, cell_is_int, get_column_count, open_asset_stream, read_cell_stream,
    read_csv_cell, read_csv_row,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct OrbGradeRow {
    pub upgrade_count: i32,
    pub sell_value: i32,
    pub remove_cost: i32,
    pub name: Vec<u8>,
}

pub fn load_orb_grade_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.orb_store.grades.clear();

    let Some(bytes) = open_asset_stream(ctx, b"equipmentgrade.csv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);

    while read_csv_row(stm) {
        if (get_column_count(stm) as i32) < 4 {
            return Ok(());
        }

        if !cell_is_int(stm, 0) {
            return Ok(());
        }

        ctx.orb_store.grades.push(OrbGradeRow::default());

        let upgrade_count = read_csv_cell(stm, 0) as i32;
        let sell_value = read_csv_cell(stm, 1) as i32;
        let remove_cost = read_csv_cell(stm, 2) as i32;
        let name = read_cell_stream(stm, 3).to_vec();

        let Some(row) = ctx.orb_store.grades.last_mut() else {
            return Ok(());
        };

        row.upgrade_count = upgrade_count;
        row.sell_value = sell_value;
        row.remove_cost = remove_cost;
        row.name = name;
    }

    Ok(())
}
