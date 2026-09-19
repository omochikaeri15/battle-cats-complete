use crate::Fault;

use super::{xor_row46_get, AppContext};

pub fn stage_reward_item(ctx: &AppContext, stage: i32, slot: i32) -> Result<i32, Fault> {
    let row = AppContext::MAP_STAGE_ROWS as i64 + (stage as i64) * AppContext::MAP_STAGE_ROW_STRIDE as i64;
    let field = slot.wrapping_mul(3).wrapping_add((slot > 0) as i32).wrapping_add(6) as i64 as usize;

    Ok(xor_row46_get(ctx.bytes_from(row as usize)?, field).ok_or(Fault::IndexOutOfRange { site: "stage_reward_item", index: field as i64, limit: 0x2e })? as i32)
}
