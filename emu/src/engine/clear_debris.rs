use crate::Fault;

use super::AppContext;

pub fn clear_debris(ctx: &mut AppContext, slot: usize) -> Result<(), Fault> {
    ctx.set_block_at::<0x10>(slot, [0; 0x10])
}
