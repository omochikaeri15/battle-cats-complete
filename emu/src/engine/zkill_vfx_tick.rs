use crate::Fault;

use super::{AppContext, VfxSlot, maanim_get_max_keyframe};

pub fn zkill_vfx_tick(ctx: &mut AppContext) -> Result<(), Fault> {
    for record in 0..0x1eusize {
        let slot =
            AppContext::ZKILL_VFX.wrapping_add(record.wrapping_mul(AppContext::ZKILL_VFX_STRIDE));

        if ctx.u8_at(slot.wrapping_add(VfxSlot::ACTIVE))? == 0 {
            continue;
        }

        let frame = ctx
            .i32_at(slot.wrapping_add(VfxSlot::FRAME))?
            .wrapping_add(1);

        ctx.set_i32_at(slot.wrapping_add(VfxSlot::FRAME), frame)?;

        if frame >= maanim_get_max_keyframe(&ctx.zkill_vfx_anim)? {
            ctx.set_block_at::<1>(slot.wrapping_add(VfxSlot::ACTIVE), [0])?;
        }
    }

    Ok(())
}
