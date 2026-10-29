use crate::Fault;

use super::AppContext;

pub fn clear_wave_sprite(ctx: &mut AppContext, slot: usize) -> Result<(), Fault> {
    ctx.set_block_at::<8>(slot, [0; 8])
}
