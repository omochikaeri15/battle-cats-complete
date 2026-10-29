use crate::Fault;

use super::{AppContext, get_global_map_id, get_map_type, get_stage_index, get_crown_level};

pub fn get_current_stage_id(ctx: &mut AppContext) -> Result<i32, Fault> {
    let leading = if get_map_type(ctx, 0)? == 0
        || get_map_type(ctx, 0)? == 1
        || get_map_type(ctx, 0)? == 2
        || get_map_type(ctx, 0)? == -9
    {
        let map_part = get_global_map_id(ctx, 0)?.wrapping_mul(0x3e8);

        get_crown_level(ctx)?
            .wrapping_mul(0x64)
            .wrapping_add(map_part)
    } else {
        get_global_map_id(ctx, 0)?.wrapping_mul(0x3e8)
    };

    Ok(get_stage_index(ctx)?.wrapping_add(leading))
}
