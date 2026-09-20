use crate::{Fault, operation};

use super::{AppContext, UNIT_BUY, UNIT_BUY_STRIDE, UnitBuy};

pub fn reward_unit_id(ctx: &AppContext, id: i32) -> Result<i32, Fault> {
    if id < 0x2710 {
        let count = ctx.event_unit_rows.len() as i32;

        if count <= 0 {
            return Ok(-1);
        }

        for row in ctx.event_unit_rows.iter().take(count as u32 as usize) {
            if *row.first().ok_or(Fault::index_out_of_range(0, 0))? == id
            {
                return row.get(2).copied().ok_or(Fault::index_out_of_range(2, row.len() as i64));
            }
        }

        return Ok(-1);
    }

    for unit in 0..0x36cusize {
        let row = ctx.bytes_from(UNIT_BUY + unit * UNIT_BUY_STRIDE)?;
        let slots = UnitBuy::KEY / 4;

        if operation::xor_row_decode(row, slots, 0x17).ok_or(Fault::index_out_of_range(0x17, slots as i64))? as i32
            == id
        {
            return Ok(unit as i32);
        }

        if operation::xor_row_decode(row, slots, 0x18).ok_or(Fault::index_out_of_range(0x18, slots as i64))? as i32
            == id
        {
            return Ok(unit as i32);
        }
    }

    Ok(-1)
}
