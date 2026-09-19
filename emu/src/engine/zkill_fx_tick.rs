use crate::Fault;

use super::{maanim_get_max_keyframe, AppContext, FxSlot};

pub fn zkill_fx_tick(ctx: &mut AppContext) -> Result<(), Fault> {
    for record in 0..0x1eusize {
        let slot = AppContext::ZKILL_FX.wrapping_add(record.wrapping_mul(AppContext::ZKILL_FX_STRIDE));

        if ctx.u8_at(slot.wrapping_add(FxSlot::ACTIVE))? == 0 {
            continue;
        }

        let frame = ctx.i32_at(slot.wrapping_add(FxSlot::FRAME))?.wrapping_add(1);

        ctx.set_i32_at(slot.wrapping_add(FxSlot::FRAME), frame)?;

        if frame >= maanim_get_max_keyframe(&ctx.zkill_fx_anim)? {
            ctx.set_block_at::<1>(slot.wrapping_add(FxSlot::ACTIVE), [0])?;
        }
    }

    Ok(())
}
