use crate::Fault;

use super::{
    AppContext, AssetStream, get_column_count, open_asset_stream, read_cell_stream, read_csv_cell,
    read_csv_row,
};

pub fn load_mission_name_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    let Some(bytes) = open_asset_stream(ctx, b"Mission_Name.csv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');

    if !read_csv_row(stm) {
        return Ok(());
    }

    loop {
        let text = read_cell_stream(stm, 1).to_vec();
        let key = read_csv_cell(stm, 0) as i32;

        ctx.mission_names.insert(key, text);

        if get_column_count(stm) as i32 >= 3 {
            let text = read_cell_stream(stm, 2).to_vec();
            let key = read_csv_cell(stm, 0) as i32;

            ctx.mission_descriptions.insert(key, text);
        }

        if !read_csv_row(stm) {
            return Ok(());
        }
    }
}
