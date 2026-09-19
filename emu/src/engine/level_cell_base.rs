use crate::Fault;

use super::{AppContext, obf_value_read};

pub fn level_cell_base(ctx: &AppContext, cell: usize) -> Result<i32, Fault> {
    let base = obf_value_read(&ctx.block_at::<8>(cell)?) as u16 as u32;

    Ok(if base < 0xc350 { base as i32 } else { 0xc350 })
}
