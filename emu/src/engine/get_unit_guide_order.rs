use crate::{Fault, ops};

use super::{AppContext, UNIT_BUY, UnitBuy};

pub fn get_unit_guide_order(ctx: &AppContext, unit_id: i32) -> Result<i32, Fault> {
    let row = (unit_id as i64) << 8;
    let value = ctx.block_at::<4>((row + (UNIT_BUY + UnitBuy::GUIDE_ORDER) as i64) as usize)?;
    let key = ctx.block_at::<4>((row + (UNIT_BUY + UnitBuy::KEY) as i64) as usize)?;
    let mut pair = [0u8; 8];

    pair[..4].copy_from_slice(&value);
    pair[4..].copy_from_slice(&key);

    Ok(
        ops::xor_row_decode(&pair, 1, 0).ok_or(Fault::index_out_of_range(0, 1))? as i32,
    )
}
