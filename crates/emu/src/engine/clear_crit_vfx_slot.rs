use crate::Fault;

use super::{AppContext, VfxSlot};

pub fn clear_crit_vfx_slot(ctx: &mut AppContext, slot: usize) -> Result<(), Fault> {
    ctx.set_block_at::<1>(slot.wrapping_add(VfxSlot::ACTIVE), [0])?;
    ctx.set_i32_at(slot.wrapping_add(VfxSlot::FRAME), 0)?;
    ctx.set_block_at::<8>(slot.wrapping_add(VfxSlot::POS_X), [0; 8])
}
