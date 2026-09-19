use crate::Fault;

use super::{
    AppContext, AssetStream, cell_is_int, map_index_of_map_id, open_asset_stream, read_csv_cell,
    read_csv_row, string_format_int,
};

pub fn labyrinth_load_floors(ctx: &mut AppContext) -> Result<(), Fault> {
    let map = ctx.i32_at(AppContext::LABYRINTH + 0x550)?;
    let name = string_format_int(ctx, b"PlayContentL_%03d.csv", map_index_of_map_id(map))?;
    let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? else {
        return Ok(());
    };
    let mut stm = AssetStream::new(&bytes, b'\n');

    for stage in 0..0x64usize {
        if !read_csv_row(&mut stm) {
            break;
        }

        if !cell_is_int(&stm, 0) {
            break;
        }

        ctx.set_i32_at(
            AppContext::LABYRINTH + 0x18 + stage * 8,
            read_csv_cell(&stm, 0) as i32,
        )?;
        ctx.set_i32_at(
            AppContext::LABYRINTH + 0x1c + stage * 8,
            read_csv_cell(&stm, 1) as i32,
        )?;
    }

    Ok(())
}
