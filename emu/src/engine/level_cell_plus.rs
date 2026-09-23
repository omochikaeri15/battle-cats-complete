use crate::Fault;

use super::{AppContext, obf_value_read};

pub fn level_cell_plus(ctx: &AppContext, cell: usize) -> Result<i32, Fault> {
    let plus = obf_value_read(&ctx.block_at::<8>(cell)?) as u16 as u32;

    Ok(if plus < 0xc350 { plus as i32 } else { 0xc350 })
}
