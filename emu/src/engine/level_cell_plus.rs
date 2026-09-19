use crate::Fault;

use super::{AppContext, obf_value_read};

pub fn level_cell_plus(ctx: &AppContext, cell: usize) -> Result<i32, Fault> {
    let packed = obf_value_read(&ctx.block_at::<8>(cell)?);

    Ok(if packed < 0xc351_0000 {
        (packed >> 0x10) as i32
    } else {
        0xc350
    })
}
