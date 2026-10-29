use crate::Fault;

use super::{
    AppContext, AssetStream, map_index_of_map_id, map_type_of_map_id, open_asset_stream,
    read_csv_cell, read_csv_row,
};

pub fn load_play_stage_limit_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.stage_conditions.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"PlayStageLimit.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            let map_id = read_csv_cell(stm, 0) as i32;
            let stage = read_csv_cell(stm, 1) as i32;
            let condition = read_csv_cell(stm, 2) as i32;
            let map_type = map_type_of_map_id(map_id);
            let map_index = map_index_of_map_id(map_id);

            *ctx.stage_conditions
                .entry(map_type)
                .or_default()
                .entry(map_index)
                .or_default()
                .entry(stage)
                .or_insert(0) = condition;
        }
    }

    Ok(())
}
