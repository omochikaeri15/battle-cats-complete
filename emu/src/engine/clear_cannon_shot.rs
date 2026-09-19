use crate::Fault;

use super::AppContext;

pub fn clear_cannon_shot(ctx: &mut AppContext, slot: usize) -> Result<(), Fault> {
    ctx.set_i32_at(slot + 8, 0)?;
    ctx.set_block_at::<8>(slot, [0; 8])
}
