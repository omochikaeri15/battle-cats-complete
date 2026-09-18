use crate::Fault;

use super::{get_map_index, get_map_type, map_type_base_id, AppContext};

pub fn get_global_map_id(ctx: &mut AppContext, base_only: u8) -> Result<i32, Fault> {
    let map_type = get_map_type(ctx, base_only)?;
    let map_idx = get_map_index(ctx, base_only)?;

    Ok(map_type_base_id(map_type, map_idx))
}
