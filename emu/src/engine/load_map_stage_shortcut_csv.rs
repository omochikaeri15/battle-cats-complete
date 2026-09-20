use crate::Fault;

use super::{
    AppContext, AssetStream, get_column_count, map_index_of_map_id, map_type_of_map_id,
    open_asset_stream, read_csv_cell, read_csv_row,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MapStageShortcut {
    pub map_type: i32,
    pub map_index: i32,
    pub stage: i32,
    pub value: i32,
    pub optional_a: i32,
    pub optional_b: i32,
    pub has_override: u8,
}

pub fn load_map_stage_shortcut_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.map_stage_shortcuts[0].clear();
    ctx.map_stage_shortcuts[1].clear();

    let Some(bytes) = open_asset_stream(ctx, b"MapStage_Shortcut.csv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) {
        let side = read_csv_cell(stm, 0) as i32;
        let key = read_csv_cell(stm, 1) as i32;
        let map_id = read_csv_cell(stm, 2) as i32;
        let map_type = map_type_of_map_id(map_id);
        let map_index = map_index_of_map_id(map_id);
        let stage = read_csv_cell(stm, 3) as i32;
        let mut value = read_csv_cell(stm, 4) as i32;
        let mut has_override = 0u8;

        if get_column_count(stm) as i32 >= 6 && read_csv_cell(stm, 5) != -1 {
            value = read_csv_cell(stm, 5) as i32;
            has_override = 1;
        }

        let mut optional_a = -1i32;

        if get_column_count(stm) as i32 >= 7 {
            optional_a = read_csv_cell(stm, 6) as i32;
        }

        let mut optional_b = -1i32;

        if get_column_count(stm) as i32 >= 8 {
            optional_b = read_csv_cell(stm, 7) as i32;
        }

        let entry = MapStageShortcut {
            map_type,
            map_index,
            stage,
            value,
            optional_a,
            optional_b,
            has_override,
        };

        if side == 1 {
            ctx.map_stage_shortcuts[1].entry(key).or_default().push(entry);
        } else if side == 0 {
            ctx.map_stage_shortcuts[0].entry(key).or_default().push(entry);
        }
    }

    Ok(())
}
