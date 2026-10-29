use crate::Fault;

use super::{AppContext, WaveRecord};

pub fn clear_effect_slot(ctx: &mut AppContext, slot: usize) -> Result<(), Fault> {
    ctx.set_block_at::<1>(slot + WaveRecord::MINI, [0])?;
    ctx.set_block_at::<0x10>(slot, [0; 0x10])?;
    ctx.set_block_at::<8>(slot + WaveRecord::POS_X, [0; 8])?;
    ctx.set_block_at::<8>(slot + WaveRecord::PROC_FLAGS, [0; 8])?;
    ctx.set_i32_at(slot + WaveRecord::PROC_EXTRA, 0)
}
