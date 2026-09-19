use crate::Fault;

use super::{AppContext, obf_value_read};

pub fn get_tech_level(ctx: &AppContext, cell: usize) -> Result<i32, Fault> {
    let packed = obf_value_read(&ctx.block_at::<8>(cell)?);
    let mut plus = packed >> 0x10;

    if packed >= 0xc3510000 {
        plus = 0xc350;
    }

    let base = obf_value_read(&ctx.block_at::<8>(cell)?) & 0xffff;
    let base = if base < 0xc350 { base } else { 0xc350 };

    Ok(base.wrapping_add(plus) as i32)
}
