use crate::Fault;

use super::{
    AppContext, AssetStream, load_gatya_data_set_csv, open_asset_stream, parse_gatya_item_row,
    read_csv_cell, read_csv_row,
};

pub fn load_gatya_ability_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.ability_data_rows = [[0; 5]; 10];

    if let Some(bytes) = open_asset_stream(ctx, b"AbilityData.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut row = 0usize;

        while row != 10 {
            read_csv_row(stm);

            let mut column = 0usize;

            while column != 5 {
                if let Some(slot) = ctx
                    .ability_data_rows
                    .get_mut(row)
                    .and_then(|cells| cells.get_mut(column))
                {
                    *slot = read_csv_cell(stm, column as i32) as i32;
                }

                column += 1;
            }

            row += 1;
        }
    }

    if let Some(bytes) = open_asset_stream(ctx, b"Gatyaitembuy.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_csv_row(stm);

        let mut index = 0usize;

        while index != 275 {
            if let Some(row) = ctx.gatya_item_rows.get_mut(index) {
                parse_gatya_item_row(row, stm);
            }

            index += 1;
        }
    }

    let mut store = Vec::new();

    load_gatya_data_set_csv(ctx, &mut store, b"R")?;
    ctx.gatya_data_sets.insert(0, store);

    let mut store = Vec::new();

    load_gatya_data_set_csv(ctx, &mut store, b"N")?;
    ctx.gatya_data_sets.insert(1, store);

    let mut store = Vec::new();

    load_gatya_data_set_csv(ctx, &mut store, b"E")?;
    ctx.gatya_data_sets.insert(2, store);

    Ok(())
}
