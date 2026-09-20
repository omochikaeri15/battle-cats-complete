use crate::Fault;

use super::{AppContext, open_asset_stream, string_format_int};

pub fn has_map_data_csv(ctx: &mut AppContext, map_index: i32) -> Result<bool, Fault> {
    let name = string_format_int(ctx, b"MapData_%03d.csv", map_index)?;

    Ok(open_asset_stream(ctx, &name, 0, 0)?.is_some())
}
