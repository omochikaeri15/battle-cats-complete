use crate::Fault;

use super::{
    AppContext, AssetStream, ShakeRecord, cell_is_int, open_asset_stream, read_csv_cell,
    read_csv_row,
};

pub fn load_base_shake_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.base_shake.records.clear();

    let Some(bytes) = open_asset_stream(ctx, b"battleshake_setting.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) {
        if !cell_is_int(stm, 0) {
            break;
        }

        let key = read_csv_cell(stm, 0) as i32;
        let record = ctx
            .base_shake
            .records
            .entry(key)
            .or_insert_with(|| ShakeRecord {
                duration: 1,
                ..ShakeRecord::default()
            });

        record.since = 0;
        record.count = 0;

        let value = read_csv_cell(stm, 1) as i32;

        ctx.base_shake.records.entry(key).or_default().amplitude_from = value;

        let value = read_csv_cell(stm, 2) as i32;

        ctx.base_shake.records.entry(key).or_default().amplitude_to = value;

        let value = read_csv_cell(stm, 3) as i32;

        ctx.base_shake.records.entry(key).or_default().duration = value;

        let value = read_csv_cell(stm, 4) as i32;

        ctx.base_shake.records.entry(key).or_default().until = value;

        let value = read_csv_cell(stm, 5) as i32;

        ctx.base_shake.records.entry(key).or_default().reset_frame = value;

        let value = read_csv_cell(stm, 6) as i32;

        ctx.base_shake.records.entry(key).or_default().priority = value;
        ctx.base_shake.records.entry(key).or_default().duration = 1;
    }

    Ok(())
}
