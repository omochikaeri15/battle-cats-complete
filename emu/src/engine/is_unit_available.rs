use crate::Fault;

use super::{AppContext, UNIT_BUY, UnitBuy};

pub fn is_unit_available(ctx: &AppContext, unit_id: i32) -> Result<bool, Fault> {
    let row = (unit_id as i64) << 8;
    let high = ctx.u8_at((row + (UNIT_BUY + UnitBuy::AVAILABLE + 3) as i64) as usize)?
        ^ ctx.u8_at((row + (UNIT_BUY + UnitBuy::KEY + 3) as i64) as usize)?;

    Ok(!high >> 7 != 0)
}
