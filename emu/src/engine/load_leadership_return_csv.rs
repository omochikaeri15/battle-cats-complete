use crate::Fault;

use super::{
    AppContext, AssetStream, cell_is_int, map_type_of_map_id, open_asset_stream, read_csv_cell,
    read_csv_row,
};

pub fn load_leadership_return_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.leadership_return_maps.clear();

    let Some(bytes) = open_asset_stream(ctx, b"LeadershipReturnMap.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) {
        if !cell_is_int(stm, 0) {
            break;
        }

        let map_id = read_csv_cell(stm, 0) as i32;
        let map_type = map_type_of_map_id(map_id);

        if map_type == -21 {
            continue;
        }

        if map_type == -11 {
            continue;
        }

        let value = read_csv_cell(stm, 1) as i32;

        ctx.leadership_return_maps
            .entry(map_id)
            .or_default()
            .push(value);
    }

    Ok(())
}
