use crate::Fault;

use super::{AppContext, FxSlot};

pub fn clear_zkill_fx_slot(ctx: &mut AppContext, slot: usize) -> Result<(), Fault> {
    ctx.set_block_at::<1>(slot.wrapping_add(FxSlot::ACTIVE), [0])?;
    ctx.set_i32_at(slot.wrapping_add(FxSlot::FRAME), 0)
}
