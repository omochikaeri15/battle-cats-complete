use crate::Fault;

use super::{AppContext, VfxSlot, maanim_get_max_keyframe};

pub fn barrier_vfx_tick(ctx: &mut AppContext) -> Result<(), Fault> {
    for record in 0..0x1eusize {
        let slot = AppContext::BARRIER_VFX
            .wrapping_add(record.wrapping_mul(AppContext::BARRIER_VFX_STRIDE));

        if ctx.u8_at(slot.wrapping_add(VfxSlot::ACTIVE))? == 0 {
            continue;
        }

        let frame = ctx
            .i32_at(slot.wrapping_add(VfxSlot::FRAME))?
            .wrapping_add(1);

        ctx.set_i32_at(slot.wrapping_add(VfxSlot::FRAME), frame)?;

        let anim = if ctx.u8_at(slot.wrapping_add(VfxSlot::BROKEN))? == 0 {
            &ctx.barrier_anims[1]
        } else {
            &ctx.barrier_anims[2]
        };

        if frame >= maanim_get_max_keyframe(anim)? {
            ctx.set_block_at::<1>(slot.wrapping_add(VfxSlot::ACTIVE), [0])?;
        }
    }

    Ok(())
}
