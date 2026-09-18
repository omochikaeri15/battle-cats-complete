use crate::Fault;

use super::{get_map_type, AppContext};

pub fn get_stage_index(ctx: &mut AppContext) -> Result<i32, Fault> {
    let map_type = get_map_type(ctx, 0)?;

    ctx.i32_at(if map_type != -8 { AppContext::STAGE_INDEX } else { 0x402170 })
}
