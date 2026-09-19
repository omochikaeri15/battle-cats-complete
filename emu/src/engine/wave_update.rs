use crate::{operation, Fault};

use super::{get_anim_len, get_battle_status, get_setting, play_sound, sound_manager, AppContext, WaveRecord, WaveSprite, CANNON_SHOT_SPACING};

const SITE: &str = "wave_update";

pub fn wave_update(ctx: &mut AppContext) -> Result<(), Fault> {
    let mut wave_index = 0usize;

    while wave_index != 200 {
        let record = AppContext::WAVE_RECORDS.wrapping_add(wave_index.wrapping_mul(AppContext::WAVE_RECORD_STRIDE));
        let sprites = AppContext::WAVE_SPRITES.wrapping_add(wave_index.wrapping_mul(AppContext::WAVE_RECORD_STRIDE));
        let mut interval = 4;

        if ctx.u8_at(record.wrapping_add(WaveRecord::MINI))? != 0 {
            interval = get_setting(&ctx.settings, b"battle_wave_s_inter", 4)?;
        }

        wave_index += 1;

        if ctx.i32_at(record.wrapping_add(WaveRecord::KIND))?.wrapping_sub(1) as u32 > 1 {
            continue;
        }

        let frame = ctx.i32_at(record.wrapping_add(WaveRecord::FRAME))?.wrapping_add(1);

        ctx.set_i32_at(record.wrapping_add(WaveRecord::FRAME), frame)?;

        if operation::irem(frame, interval).ok_or(Fault::divide(SITE, interval as i64))? != 0 {
            continue;
        }

        let mut sprite = 0usize;

        while sprite != 6 && ctx.i32_at(sprites.wrapping_add(sprite.wrapping_mul(WaveSprite::STRIDE)).wrapping_add(WaveSprite::TIMER))? != 0 {
            sprite += 1;
        }

        if sprite == 6 {
            continue;
        }

        if get_battle_status(ctx)? == 0 {
            play_sound(sound_manager(ctx)?, 0x1a, None);
        }

        let length = if ctx.u8_at(record.wrapping_add(WaveRecord::MINI))? != 0 { get_anim_len(&ctx.mini_wave_anim)? } else { get_anim_len(&ctx.wave_anim)? };

        ctx.set_i32_at(sprites.wrapping_add(sprite.wrapping_mul(WaveSprite::STRIDE)).wrapping_add(WaveSprite::TIMER), length.wrapping_sub(1))?;

        let step;

        match ctx.i32_at(record.wrapping_add(WaveRecord::KIND))? {
            1 => {
                ctx.set_i32_at(record.wrapping_add(WaveRecord::IN_USE), 1)?;

                let frame = ctx.i32_at(record.wrapping_add(WaveRecord::FRAME))?;
                let origin = ctx.i32_at(record.wrapping_add(WaveRecord::POS_X))?;

                step = operation::idiv(frame, interval).ok_or(Fault::divide(SITE, interval as i64))?;

                let x = origin.wrapping_add(step.wrapping_mul(CANNON_SHOT_SPACING).wrapping_mul(5).wrapping_neg()).wrapping_add(0x10e);

                ctx.set_i32_at(sprites.wrapping_add(sprite.wrapping_mul(WaveSprite::STRIDE)).wrapping_add(WaveSprite::POS_X), x)?;
            }
            2 => {
                ctx.set_i32_at(record.wrapping_add(WaveRecord::IN_USE), 2)?;

                let frame = ctx.i32_at(record.wrapping_add(WaveRecord::FRAME))?;
                let origin = ctx.i32_at(record.wrapping_add(WaveRecord::POS_X))?;

                step = operation::idiv(frame, interval).ok_or(Fault::divide(SITE, interval as i64))?;

                let unit = CANNON_SHOT_SPACING.wrapping_mul(5);
                let x = origin.wrapping_sub(unit).wrapping_add(step.wrapping_mul(unit)).wrapping_add(0x10e);

                ctx.set_i32_at(sprites.wrapping_add(sprite.wrapping_mul(WaveSprite::STRIDE)).wrapping_add(WaveSprite::POS_X), x)?;
            }
            _ => {
                let frame = ctx.i32_at(record.wrapping_add(WaveRecord::FRAME))?;

                step = operation::idiv(frame, interval).ok_or(Fault::divide(SITE, interval as i64))?;
            }
        }

        if step >= ctx.i32_at(record.wrapping_add(WaveRecord::LEVEL))? {
            ctx.set_i32_at(record.wrapping_add(WaveRecord::KIND), 0)?;
            ctx.set_i32_at(record.wrapping_add(WaveRecord::FRAME), 0)?;
        }
    }

    let mut wave_index = 0usize;

    while wave_index != 200 {
        let record = AppContext::WAVE_RECORDS.wrapping_add(wave_index.wrapping_mul(AppContext::WAVE_RECORD_STRIDE));
        let sprites = AppContext::WAVE_SPRITES.wrapping_add(wave_index.wrapping_mul(AppContext::WAVE_RECORD_STRIDE));
        let mut sprite = 0usize;

        while sprite != 6 && ctx.i32_at(sprites.wrapping_add(sprite.wrapping_mul(WaveSprite::STRIDE)).wrapping_add(WaveSprite::TIMER))? <= 0 {
            sprite += 1;
        }

        if sprite == 6 && ctx.i32_at(record.wrapping_add(WaveRecord::KIND))? == 0 && ctx.i32_at(record.wrapping_add(WaveRecord::IN_USE))? != 0 {
            ctx.set_i32_at(record.wrapping_add(WaveRecord::IN_USE), 0)?;
        }

        wave_index += 1;
    }

    Ok(())
}
