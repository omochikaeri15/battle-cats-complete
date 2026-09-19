use crate::{operation, Fault};

use super::{AppContext, UnitBuy, UNIT_BUY};

pub fn get_unit_max_plus_level(ctx: &AppContext, unit_id: i32) -> Result<i32, Fault> {
    let row = (unit_id as i64) << 8;
    let mut pair = [0u8; 8];

    pair[..4].copy_from_slice(&ctx.block_at::<4>((row + (UNIT_BUY + UnitBuy::MAX_PLUS_LEVEL) as i64) as usize)?);
    pair[4..].copy_from_slice(&ctx.block_at::<4>((row + (UNIT_BUY + UnitBuy::KEY) as i64) as usize)?);

    Ok(operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::IndexOutOfRange { site: "get_unit_max_plus_level", index: 0, limit: 1 })? as i32)
}
