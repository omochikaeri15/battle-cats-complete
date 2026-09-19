use crate::Fault;

use super::{clear_crit_fx_slot, maanim_get_max_keyframe, play_sound_in_battle, AppContext, FxSlot};

pub fn crit_fx_tick(ctx: &mut AppContext) -> Result<(), Fault> {
    for record in 0..0xc8usize {
        let slot = AppContext::CRIT_FX.wrapping_add(record.wrapping_mul(AppContext::CRIT_FX_STRIDE));

        if ctx.u8_at(slot.wrapping_add(FxSlot::ACTIVE))? == 0 {
            continue;
        }

        let mut frame = ctx.i32_at(slot.wrapping_add(FxSlot::FRAME))?;

        if frame == 0 {
            play_sound_in_battle(ctx, 0x2c)?;
            frame = ctx.i32_at(slot.wrapping_add(FxSlot::FRAME))?;
        }

        frame = frame.wrapping_add(1);
        ctx.set_i32_at(slot.wrapping_add(FxSlot::FRAME), frame)?;

        if frame >= maanim_get_max_keyframe(&ctx.crit_fx_anim)? {
            clear_crit_fx_slot(ctx, slot)?;
        }
    }

    Ok(())
}
