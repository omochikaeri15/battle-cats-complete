use crate::Fault;

use super::{
    AppContext, AssetStream, open_asset_stream, read_csv_cell, read_csv_cell_float, read_csv_row,
};

#[derive(Clone, Default, PartialEq, Debug)]
pub struct DropItemRow {
    pub map_id: i32,
    pub crown_rates: Vec<f32>,
    pub stage_rates: Vec<f32>,
    pub miss_weight: i32,
    pub item_weights: Vec<i32>,
}

pub fn load_drop_item_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.drop_item_rows.clear();

    let Some(bytes) = open_asset_stream(ctx, b"DropItem.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);

    while read_csv_row(stm) {
        let key = read_csv_cell(stm, 0) as i32;
        let row = ctx.drop_item_rows.entry(key).or_default();

        row.map_id = 0;
        row.crown_rates.clear();
        row.stage_rates.clear();
        row.miss_weight = 0;
        row.item_weights.clear();
        ctx.drop_item_rows.entry(key).or_default().map_id = key;

        let mut column = 0i32;

        while column != 4 {
            column += 1;

            let value = read_csv_cell_float(stm, column);

            ctx.drop_item_rows
                .entry(key)
                .or_default()
                .crown_rates
                .push(value);
        }

        let mut stage = 0i32;

        while stage != 8 {
            let value = read_csv_cell_float(stm, stage.wrapping_add(5));

            ctx.drop_item_rows
                .entry(key)
                .or_default()
                .stage_rates
                .push(value);
            stage += 1;
        }

        let value = read_csv_cell(stm, 0xd) as i32;

        ctx.drop_item_rows.entry(key).or_default().miss_weight = value;

        let mut item = 0i32;

        while item != 0x10 {
            let value = read_csv_cell(stm, item.wrapping_add(0xe)) as i32;

            ctx.drop_item_rows
                .entry(key)
                .or_default()
                .item_weights
                .push(value);
            item += 1;
        }
    }

    Ok(())
}
