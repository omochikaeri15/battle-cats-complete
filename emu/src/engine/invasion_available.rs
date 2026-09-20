use crate::{Fault, operation};

use super::AppContext;

pub fn invasion_available(ctx: &AppContext, chapter: i32) -> Result<bool, Fault> {
    if chapter != 9 {
        return Ok(false);
    }

    let cleared = operation::xor_row_decode(
        &ctx.block_at::<8>(AppContext::STAGES_CLEARED_CHAPTERS + 9 * 4)?,
        1,
        0,
    )
    .ok_or(Fault::index_out_of_range(0, 1))? as i32;

    if cleared < 0x30 {
        return Ok(false);
    }

    Ok(ctx.u8_at(AppContext::MAP_NEG15_CLEARED)? == 0)
}
