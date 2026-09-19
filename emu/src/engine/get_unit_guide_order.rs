use crate::{operation, Fault};

use super::{AppContext, UnitBuy, UNIT_BUY};

pub fn get_unit_guide_order(ctx: &AppContext, unit_id: i32) -> Result<i32, Fault> {
    let row = (unit_id as i64) << 8;
    let value = ctx.block_at::<4>((row + (UNIT_BUY + UnitBuy::GUIDE_ORDER) as i64) as usize)?;
    let key = ctx.block_at::<4>((row + (UNIT_BUY + UnitBuy::KEY) as i64) as usize)?;
    let mut pair = [0u8; 8];

    pair[..4].copy_from_slice(&value);
    pair[4..].copy_from_slice(&key);

    Ok(operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::IndexOutOfRange { site: "get_unit_guide_order", index: 0, limit: 1 })? as i32)
}
