use crate::Fault;

use super::AppContext;

pub fn clear_crit_fx(ctx: &mut AppContext, slot: usize) -> Result<(), Fault> {
    ctx.set_block_at::<1>(slot, [0])?;
    ctx.set_i32_at(slot + 0xc, 0)?;
    ctx.set_block_at::<8>(slot + 4, [0; 8])
}
