use crate::Fault;

use super::{AppContext, obf_value_read, obf_value_set};

pub fn level_cell_add(ctx: &mut AppContext, cell: usize, delta: i32) -> Result<(), Fault> {
    let packed = obf_value_read(&ctx.block_at::<8>(cell)?);
    let plus = if packed >= 0xc3510000 {
        0xc350
    } else {
        packed >> 0x10
    };
    let base = obf_value_read(&ctx.block_at::<8>(cell)?) as u16 as i32;
    let base = if base >= 0xc350 { 0xc350 } else { base }.wrapping_add(delta);
    let high = if plus < 0xc351 {
        plus << 0x10
    } else {
        0xc3500000
    };
    let base = if base > 0 { base } else { 0 };
    let base = if base >= 0xc350 { 0xc350 } else { base };
    let mut value = ctx.block_at::<8>(cell)?;

    obf_value_set(&mut value, high | base as u32);
    ctx.set_block_at::<8>(cell, value)?;
    ctx.units_dirty = 1;

    Ok(())
}
