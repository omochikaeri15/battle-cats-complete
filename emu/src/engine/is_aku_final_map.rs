use crate::Fault;

use super::{get_map_index, get_map_type, AppContext};

pub fn is_aku_final_map(ctx: &mut AppContext) -> Result<bool, Fault> {
    if get_map_type(ctx, 0)? != -8 {
        return Ok(false);
    }

    Ok(get_map_index(ctx, 0)? == 0x2a)
}
