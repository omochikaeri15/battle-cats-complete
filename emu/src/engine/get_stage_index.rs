use crate::Fault;

use super::{AppContext, get_map_type};

pub fn get_stage_index(ctx: &mut AppContext) -> Result<i32, Fault> {
    let map_type = get_map_type(ctx, 0)?;

    ctx.i32_at(if map_type != -8 {
        AppContext::STAGE_INDEX
    } else {
        AppContext::EX_STAGE_INDEX
    })
}
