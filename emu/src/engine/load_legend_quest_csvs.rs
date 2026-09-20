use crate::Fault;

use super::{
    AppContext, AssetStream, get_map_index, get_map_type, open_asset_stream, read_cell_stream,
    read_csv_cell, read_csv_row, std_to_string, string_format_int,
};

pub fn load_legend_quest_csvs(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.random_dungeon_rows.clear();
    ctx.legend_stage_conditions.clear();

    if get_map_type(ctx, 0)? != -11 {
        return Ok(());
    }

    let map_index = get_map_index(ctx, 0)?;
    let name = string_format_int(ctx, b"RandomDungeon_%03d.csv", map_index)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            if read_cell_stream(stm, 0) != std_to_string(read_csv_cell(stm, 0) as i32).as_slice() {
                break;
            }

            ctx.random_dungeon_rows.push([
                read_csv_cell(stm, 0) as i32,
                read_csv_cell(stm, 1) as i32,
                read_csv_cell(stm, 2) as i32,
                read_csv_cell(stm, 3) as i32,
                read_csv_cell(stm, 4) as i32,
                read_csv_cell(stm, 5) as i32,
                read_csv_cell(stm, 6) as i32,
                read_csv_cell(stm, 7) as i32,
                read_csv_cell(stm, 8) as i32,
                read_csv_cell(stm, 9) as i32,
                read_csv_cell(stm, 0xa) as i32,
            ]);
        }
    }

    let map_index = get_map_index(ctx, 0)?;
    let name = string_format_int(ctx, b"PlayDungeonD_%03d.csv", map_index)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut stage = 0i64;

        while read_csv_row(stm) {
            ctx.legend_stage_conditions.push([
                read_csv_cell(stm, 0) as i32,
                read_csv_cell(stm, 1) as i32,
                read_csv_cell(stm, 2) as i32,
                read_csv_cell(stm, 3) as i32,
                read_csv_cell(stm, 4) as i32,
                read_csv_cell(stm, 5) as i32,
                read_csv_cell(stm, 6) as i32,
                read_csv_cell(stm, 7) as i32,
                i32::from(read_csv_cell(stm, 8) != 0),
            ]);

            stage += 1;

            if stage == 0x30 {
                break;
            }
        }
    }

    Ok(())
}
