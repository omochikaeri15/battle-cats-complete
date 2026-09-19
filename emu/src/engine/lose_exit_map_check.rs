use crate::Fault;

use super::{AppContext, get_map_index, get_map_type};

pub fn lose_exit_map_check(ctx: &mut AppContext) -> Result<bool, Fault> {
    if get_map_type(ctx, 0)? == -19 {
        return Ok(true);
    }

    if get_map_type(ctx, 0)? != -8 {
        return Ok(false);
    }

    Ok(get_map_index(ctx, 0)? == 0x2a)
}
