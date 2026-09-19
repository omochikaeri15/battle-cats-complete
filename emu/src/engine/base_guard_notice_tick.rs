use crate::Fault;

use super::{AppContext, get_anim_len, is_boss_guarding_base, play_sound, sound_manager};

pub fn base_guard_notice_tick(ctx: &mut AppContext) -> Result<(), Fault> {
    let guarding = is_boss_guarding_base(ctx)?;
    let state = ctx.i32_at(AppContext::BASE_GUARD_NOTICE)?;

    if guarding && state == 0 {
        ctx.set_i32_at(AppContext::BASE_GUARD_NOTICE, 2)?;
        ctx.set_i32_at(AppContext::BASE_GUARD_NOTICE_FRAME, 1)?;

        return Ok(());
    }

    if (state.wrapping_sub(1) as u32) <= 1
        && !(guarding || ctx.i32_at(AppContext::BASE_GUARD_NOTICE_FRAME)? <= 0)
    {
        ctx.set_i32_at(AppContext::BASE_GUARD_NOTICE, 3)?;
        ctx.set_i32_at(AppContext::BASE_GUARD_NOTICE_FRAME, 1)?;
        play_sound(sound_manager(ctx)?, 0x46, None);

        return Ok(());
    }

    if state == 3 {
        let frame = ctx.i32_at(AppContext::BASE_GUARD_NOTICE_FRAME)?;
        let next = frame.wrapping_add(1);

        ctx.set_i32_at(AppContext::BASE_GUARD_NOTICE_FRAME, next)?;

        if frame == 0 {
            play_sound(sound_manager(ctx)?, 0x46, None);

            return Ok(());
        }

        if next >= get_anim_len(&ctx.barrier_anims[1])? {
            ctx.set_i32_at(AppContext::BASE_GUARD_NOTICE, 0)?;
        }

        return Ok(());
    }

    if state == 1 {
        let frame = ctx.i32_at(AppContext::BASE_GUARD_NOTICE_FRAME)?;
        let next = frame.wrapping_add(1);

        ctx.set_i32_at(AppContext::BASE_GUARD_NOTICE_FRAME, next)?;

        if frame == 0 {
            play_sound(sound_manager(ctx)?, 0x47, None);

            return Ok(());
        }

        if next >= get_anim_len(&ctx.barrier_anims[0])? {
            ctx.set_i32_at(AppContext::BASE_GUARD_NOTICE, 2)?;
        }
    }

    Ok(())
}
