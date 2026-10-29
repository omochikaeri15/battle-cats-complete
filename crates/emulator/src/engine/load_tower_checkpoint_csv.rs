use crate::Fault;

use super::{
    AppContext, AssetStream, StagePairRecord, cell_is_int, open_asset_stream, read_cell_stream,
    read_csv_cell, read_csv_row,
};

pub fn load_tower_checkpoint_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.stage_pair_records.clear();

    let Some(bytes) = open_asset_stream(ctx, b"Stageskip_setting.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) {
        if !cell_is_int(stm, 0) {
            break;
        }

        let map = read_csv_cell(stm, 0) as i32;
        let stage = read_csv_cell(stm, 1) as i32;
        let key = stage.wrapping_add(map.wrapping_mul(100));

        ctx.stage_pair_records
            .insert(key, StagePairRecord::default());

        let value = read_csv_cell(stm, 2) as i32;

        if let Some(record) = ctx.stage_pair_records.get_mut(&key) {
            record.other_stage = value;
        }

        let text = read_cell_stream(stm, 3).to_vec();

        if let Some(record) = ctx.stage_pair_records.get_mut(&key) {
            record.message = text;
        }
    }

    Ok(())
}
