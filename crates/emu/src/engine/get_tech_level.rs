use crate::Fault;

use super::{AppContext, obf_value_read};

pub fn get_tech_level(ctx: &AppContext, cell: usize) -> Result<i32, Fault> {
    let packed = obf_value_read(&ctx.block_at::<8>(cell)?);
    let mut base = packed >> 0x10;

    if packed >= 0xc3510000 {
        base = 0xc350;
    }

    let plus = obf_value_read(&ctx.block_at::<8>(cell)?) & 0xffff;
    let plus = if plus < 0xc350 { plus } else { 0xc350 };

    Ok(plus.wrapping_add(base) as i32)
}
