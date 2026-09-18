use crate::{operation, Fault};

use super::AppContext;

pub fn invasion_available(ctx: &AppContext, chapter: i32) -> Result<bool, Fault> {
    if chapter != 9 {
        return Ok(false);
    }

    let cleared = operation::xor_row_decode(&ctx.block_at::<8>(0xc970)?, 1, 0)
        .ok_or(Fault::IndexOutOfRange { site: "invasion_available", index: 0, limit: 1 })? as i32;

    if cleared < 0x30 {
        return Ok(false);
    }

    Ok(ctx.u8_at(0x32c5d9)? == 0)
}
