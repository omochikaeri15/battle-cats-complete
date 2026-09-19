use crate::Fault;

use super::AppContext;

pub fn clear_effect_slot(ctx: &mut AppContext, slot: usize) -> Result<(), Fault> {
    ctx.set_block_at::<1>(slot + 0x2c, [0])?;
    ctx.set_block_at::<0x10>(slot, [0; 0x10])?;
    ctx.set_block_at::<8>(slot + 0x10, [0; 8])?;
    ctx.set_block_at::<8>(slot + 0x1c, [0; 8])?;
    ctx.set_i32_at(slot + 0x24, 0)
}
