use crate::Fault;

use super::{
    AppContext, VfxSlot, clear_crit_vfx_slot, maanim_get_max_keyframe, play_sound_in_battle,
};

pub fn crit_vfx_tick(ctx: &mut AppContext) -> Result<(), Fault> {
    for record in 0..0xc8usize {
        let slot =
            AppContext::CRIT_VFX.wrapping_add(record.wrapping_mul(AppContext::CRIT_VFX_STRIDE));

        if ctx.u8_at(slot.wrapping_add(VfxSlot::ACTIVE))? == 0 {
            continue;
        }

        let mut frame = ctx.i32_at(slot.wrapping_add(VfxSlot::FRAME))?;

        if frame == 0 {
            play_sound_in_battle(ctx, 0x2c)?;
            frame = ctx.i32_at(slot.wrapping_add(VfxSlot::FRAME))?;
        }

        frame = frame.wrapping_add(1);
        ctx.set_i32_at(slot.wrapping_add(VfxSlot::FRAME), frame)?;

        if frame >= maanim_get_max_keyframe(&ctx.crit_vfx_anim)? {
            clear_crit_vfx_slot(ctx, slot)?;
        }
    }

    Ok(())
}
