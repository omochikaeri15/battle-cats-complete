use crate::Fault;

use super::{AppContext, eoc_progress_total};

pub fn defeat_counter_bump(ctx: &mut AppContext) -> Result<(), Fault> {
    let count = ctx.i8_at(AppContext::DEFEAT_COUNTER)?;

    if count > 0x63 {
        return Ok(());
    }

    if eoc_progress_total(ctx)? >= 100 {
        ctx.set_block_at::<1>(AppContext::DEFEAT_COUNTER, [(count as u8).wrapping_add(1)])?;
    }

    Ok(())
}
