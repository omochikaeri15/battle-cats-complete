use std::collections::BTreeMap;

use crate::Fault;

use super::{
    get_anim_len, get_entity_base_idx, get_pos_x, get_setting, get_wave_block, get_wave_hit_faction, is_touchable, wave_hit_cat_side, wave_hit_enemy_side,
    AppContext, Entity, WaveRecord, WaveSprite, CANNON_SHOT_SPACING,
};

const SITE: &str = "wave_apply_hits";

pub fn wave_apply_hits(ctx: &mut AppContext) -> Result<(), Fault> {
    let mut blockers: BTreeMap<i32, i32> = BTreeMap::new();
    let mut hits: Vec<(i32, i32)> = Vec::new();
    let mut wave_index = 0usize;

    while wave_index != 200 {
        let record = AppContext::WAVE_RECORDS.wrapping_add(wave_index.wrapping_mul(AppContext::WAVE_RECORD_STRIDE));
        let sprites = AppContext::WAVE_SPRITES.wrapping_add(wave_index.wrapping_mul(AppContext::WAVE_RECORD_STRIDE));
        let mut sprite = 0usize;

        while sprite != 6 {
            let sprite_cell = sprites.wrapping_add(sprite.wrapping_mul(WaveSprite::STRIDE));
            let timer = ctx.i32_at(sprite_cell.wrapping_add(WaveSprite::TIMER))?;

            sprite += 1;

            if timer <= 0 {
                continue;
            }

            ctx.set_i32_at(sprite_cell.wrapping_add(WaveSprite::TIMER), timer.wrapping_sub(1))?;

            let length = if ctx.u8_at(record.wrapping_add(WaveRecord::MINI))? != 0 { get_anim_len(&ctx.mini_wave_anim)? } else { get_anim_len(&ctx.wave_anim)? };
            let timer = ctx.i32_at(sprite_cell.wrapping_add(WaveSprite::TIMER))?;
            let mut last_frame = 7;
            let first_frame;

            if ctx.u8_at(record.wrapping_add(WaveRecord::MINI))? != 0 {
                first_frame = get_setting(&ctx.settings, b"battle_wave_s_frame1", 3)?;

                if ctx.u8_at(record.wrapping_add(WaveRecord::MINI))? != 0 {
                    last_frame = get_setting(&ctx.settings, b"battle_wave_s_frame2", 7)?;
                }
            } else {
                first_frame = 3;
            }

            if length.wrapping_add(!timer) <= first_frame || length.wrapping_add(!timer).wrapping_sub(1) > last_frame {
                continue;
            }

            let mut slot = 1i32;

            while slot != 51 {
                if (ctx.i32_at(AppContext::entity_field(1, slot, Entity::OCCUPANT))? == 0 && ctx.i32_at(record.wrapping_add(WaveRecord::IN_USE))? == 1)
                    || (ctx.i32_at(AppContext::entity_field(0, slot, Entity::OCCUPANT))? == 0 && ctx.i32_at(record.wrapping_add(WaveRecord::IN_USE))? == 2)
                {
                    slot += 1;

                    continue;
                }

                let sprite_x;
                let x;

                if is_touchable(ctx, 1, slot, ctx.i32_at(record.wrapping_add(WaveRecord::OWNER_SLOT))?)?
                    && ctx.i32_at(record.wrapping_add(WaveRecord::IN_USE))? == 1
                    && ctx.u8_at(AppContext::WAVE_HITS.wrapping_add((slot as usize).wrapping_mul(200)).wrapping_add(wave_index))? == 0
                {
                    sprite_x = ctx.i32_at(sprite_cell.wrapping_add(WaveSprite::POS_X))?;
                    x = get_pos_x(ctx, 1, slot)?.wrapping_sub(ctx.i32_at(AppContext::entity_field(1, slot, Entity::HITBOX_POS))?);
                } else if is_touchable(ctx, 0, slot, ctx.i32_at(record.wrapping_add(WaveRecord::OWNER_SLOT))?)?
                    && ctx.i32_at(record.wrapping_add(WaveRecord::IN_USE))? == 2
                    && ctx.u8_at(AppContext::WAVE_HITS.wrapping_add((slot as usize).wrapping_mul(200)).wrapping_add(wave_index))? == 0
                {
                    sprite_x = ctx.i32_at(sprite_cell.wrapping_add(WaveSprite::POS_X))?;
                    x = get_pos_x(ctx, 0, slot)?
                        .wrapping_add(ctx.i32_at(AppContext::entity_field(0, slot, Entity::HITBOX_POS))?)
                        .wrapping_sub(CANNON_SHOT_SPACING.wrapping_mul(5));
                } else {
                    slot += 1;

                    continue;
                }

                let half = CANNON_SHOT_SPACING.wrapping_mul(5);

                if x < sprite_x.wrapping_sub(half)
                    || x >= half.wrapping_add(sprite_x)
                    || ctx.u8_at(AppContext::WAVE_HITS.wrapping_add((slot as usize).wrapping_mul(200)).wrapping_add(wave_index))? != 0
                {
                    slot += 1;

                    continue;
                }

                match ctx.i32_at(record.wrapping_add(WaveRecord::IN_USE))? {
                    2 if ctx.i32_at(AppContext::entity_field(0, slot, Entity::WAVE_BLOCK))? == 1 => {
                        ctx.set_i32_at(record.wrapping_add(WaveRecord::KIND), 0)?;
                        ctx.set_i32_at(record.wrapping_add(WaveRecord::FRAME), 0)?;
                        ctx.set_i32_at(record.wrapping_add(WaveRecord::LEVEL), 0)?;

                        let mut cleared = 0usize;

                        while cleared != 6 {
                            ctx.set_i32_at(sprites.wrapping_add(cleared.wrapping_mul(WaveSprite::STRIDE)).wrapping_add(WaveSprite::TIMER), 0)?;
                            cleared += 1;
                        }

                        ctx.set_i32_at(AppContext::entity_field(0, slot, Entity::WAVE_BLOCK_FX_FRAME), 0)?;
                        ctx.set_i32_at(AppContext::entity_field(0, slot, Entity::WAVE_BLOCK_FX_ACTIVE), 1)?;
                    }
                    1 if ctx.i32_at(AppContext::entity_field(1, slot, Entity::WAVE_BLOCK))? == 1 => {
                        ctx.set_i32_at(record.wrapping_add(WaveRecord::KIND), 0)?;
                        ctx.set_i32_at(record.wrapping_add(WaveRecord::FRAME), 0)?;
                        ctx.set_i32_at(record.wrapping_add(WaveRecord::LEVEL), 0)?;

                        let mut cleared = 0usize;

                        while cleared != 6 {
                            ctx.set_i32_at(sprites.wrapping_add(cleared.wrapping_mul(WaveSprite::STRIDE)).wrapping_add(WaveSprite::TIMER), 0)?;
                            cleared += 1;
                        }

                        ctx.set_i32_at(AppContext::entity_field(1, slot, Entity::WAVE_BLOCK_FX_FRAME), 0)?;
                        ctx.set_i32_at(AppContext::entity_field(1, slot, Entity::WAVE_BLOCK_FX_ACTIVE), 1)?;
                    }
                    _ => {}
                }

                slot += 1;
            }
        }

        wave_index += 1;
    }

    let mut wave_index = 0usize;

    while wave_index != 200 {
        let record = AppContext::WAVE_RECORDS.wrapping_add(wave_index.wrapping_mul(AppContext::WAVE_RECORD_STRIDE));
        let sprites = AppContext::WAVE_SPRITES.wrapping_add(wave_index.wrapping_mul(AppContext::WAVE_RECORD_STRIDE));
        let mut sprite = 0usize;

        while sprite != 6 {
            let sprite_cell = sprites.wrapping_add(sprite.wrapping_mul(WaveSprite::STRIDE));

            sprite += 1;

            if ctx.i32_at(sprite_cell.wrapping_add(WaveSprite::TIMER))? <= 0 {
                continue;
            }

            let length = if ctx.u8_at(record.wrapping_add(WaveRecord::MINI))? != 0 { get_anim_len(&ctx.mini_wave_anim)? } else { get_anim_len(&ctx.wave_anim)? };
            let timer = ctx.i32_at(sprite_cell.wrapping_add(WaveSprite::TIMER))?;
            let mut last_frame = 7;

            if ctx.u8_at(record.wrapping_add(WaveRecord::MINI))? != 0 {
                last_frame = get_setting(&ctx.settings, b"battle_wave_s_frame2", 7)?;
            }

            if length.wrapping_add(!timer).wrapping_sub(1) != last_frame {
                continue;
            }

            let mut slot = 1i32;

            while slot != 51 {
                if (ctx.i32_at(AppContext::entity_field(1, slot, Entity::OCCUPANT))? == 0 || get_entity_base_idx(ctx)? as u32 as u64 == slot as u64)
                    && ctx.i32_at(record.wrapping_add(WaveRecord::IN_USE))? == 1
                {
                    slot += 1;

                    continue;
                }

                if ctx.i32_at(AppContext::entity_field(0, slot, Entity::OCCUPANT))? == 0 && ctx.i32_at(record.wrapping_add(WaveRecord::IN_USE))? == 2 {
                    slot += 1;

                    continue;
                }

                let sprite_x;
                let x;

                if is_touchable(ctx, 1, slot, ctx.i32_at(record.wrapping_add(WaveRecord::OWNER_SLOT))?)?
                    && ctx.i32_at(record.wrapping_add(WaveRecord::IN_USE))? == 1
                    && ctx.u8_at(AppContext::WAVE_HITS.wrapping_add((slot as usize).wrapping_mul(200)).wrapping_add(wave_index))? == 0
                {
                    sprite_x = ctx.i32_at(sprite_cell.wrapping_add(WaveSprite::POS_X))?;
                    x = get_pos_x(ctx, 1, slot)?.wrapping_sub(ctx.i32_at(AppContext::entity_field(1, slot, Entity::HITBOX_POS))?);
                } else if is_touchable(ctx, 0, slot, ctx.i32_at(record.wrapping_add(WaveRecord::OWNER_SLOT))?)?
                    && ctx.i32_at(record.wrapping_add(WaveRecord::IN_USE))? == 2
                    && ctx.u8_at(AppContext::WAVE_HITS.wrapping_add((slot as usize).wrapping_mul(200)).wrapping_add(wave_index))? == 0
                {
                    sprite_x = ctx.i32_at(sprite_cell.wrapping_add(WaveSprite::POS_X))?;
                    x = get_pos_x(ctx, 0, slot)?
                        .wrapping_add(ctx.i32_at(AppContext::entity_field(0, slot, Entity::HITBOX_POS))?)
                        .wrapping_sub(CANNON_SHOT_SPACING.wrapping_mul(5));
                } else {
                    slot += 1;

                    continue;
                }

                let half = CANNON_SHOT_SPACING.wrapping_mul(5);

                if x < sprite_x.wrapping_sub(half)
                    || x >= half.wrapping_add(sprite_x)
                    || ctx.u8_at(AppContext::WAVE_HITS.wrapping_add((slot as usize).wrapping_mul(200)).wrapping_add(wave_index))? != 0
                {
                    slot += 1;

                    continue;
                }

                let side = match ctx.i32_at(record.wrapping_add(WaveRecord::IN_USE))? {
                    1 => Some(1),
                    2 => Some(0),
                    _ => None,
                };

                if let Some(side) = side
                    && get_wave_block(ctx, side, slot)?
                {
                    let next = hits.len() as i32;

                    if blockers.contains_key(&(wave_index as i32)) {
                        let x = get_pos_x(ctx, side, slot)?;
                        let entry = *blockers.entry(wave_index as i32).or_insert(0);
                        let blocker = hits.get(entry as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: entry as i64, limit: hits.len() as i64 })?;
                        let faction = get_wave_hit_faction(ctx, blocker.0)?;
                        let entry = *blockers.entry(wave_index as i32).or_insert(0);
                        let blocker = hits.get(entry as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: entry as i64, limit: hits.len() as i64 })?;

                        if x < get_pos_x(ctx, faction, blocker.1)? {
                            *blockers.entry(wave_index as i32).or_insert(0) = next;
                        }
                    } else {
                        *blockers.entry(wave_index as i32).or_insert(0) = next;
                    }
                }

                hits.push((wave_index as i32, slot));
                slot += 1;
            }
        }

        wave_index += 1;
    }

    let mut index = 0usize;

    while index != hits.len() {
        let (wave, slot) = *hits.get(index).ok_or(Fault::IndexOutOfRange { site: SITE, index: index as i64, limit: hits.len() as i64 })?;
        let record = AppContext::WAVE_RECORDS.wrapping_add((wave as i64).wrapping_mul(AppContext::WAVE_RECORD_STRIDE as i64) as usize);

        index += 1;

        if blockers.contains_key(&wave) {
            let entry = *blockers.entry(wave).or_insert(0);
            let blocker = hits.get(entry as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: entry as i64, limit: hits.len() as i64 })?;

            if blocker.1 != slot {
                match ctx.i32_at(record.wrapping_add(WaveRecord::IN_USE))? {
                    2 if get_pos_x(ctx, 0, slot)? > ctx.i32_at(record.wrapping_add(WaveRecord::POS_X))? => continue,
                    1 if get_pos_x(ctx, 1, slot)? < ctx.i32_at(record.wrapping_add(WaveRecord::POS_X))? => continue,
                    _ => {}
                }
            }
        }

        ctx.set_block_at::<1>(AppContext::WAVE_HITS.wrapping_add((slot as i64).wrapping_mul(200) as usize).wrapping_add(wave as i64 as usize), [1])?;

        match ctx.i32_at(record.wrapping_add(WaveRecord::IN_USE))? {
            2 => wave_hit_enemy_side(ctx, wave, slot, ctx.i32_at(record.wrapping_add(WaveRecord::ATTACK))?)?,
            1 => wave_hit_cat_side(ctx, wave, slot, ctx.i32_at(record.wrapping_add(WaveRecord::ATTACK))?)?,
            _ => {}
        }
    }

    Ok(())
}
