use crate::Fault;

use super::{AppContext, obf_value_read, obf_value_set};

pub fn level_cell_add(ctx: &mut AppContext, cell: usize, delta: i32) -> Result<(), Fault> {
    let packed = obf_value_read(&ctx.block_at::<8>(cell)?);
    let base = if packed >= 0xc3510000 {
        0xc350
    } else {
        packed >> 0x10
    };
    let plus = obf_value_read(&ctx.block_at::<8>(cell)?) as u16 as i32;
    let plus = if plus >= 0xc350 { 0xc350 } else { plus }.wrapping_add(delta);
    let high = if base < 0xc351 {
        base << 0x10
    } else {
        0xc3500000
    };
    let plus = if plus > 0 { plus } else { 0 };
    let plus = if plus >= 0xc350 { 0xc350 } else { plus };
    let mut value = ctx.block_at::<8>(cell)?;

    obf_value_set(&mut value, high | plus as u32);
    ctx.set_block_at::<8>(cell, value)?;
    ctx.units_dirty = 1;

    Ok(())
}
