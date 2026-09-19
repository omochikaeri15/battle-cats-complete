use std::collections::BTreeMap;

use crate::Fault;

use super::{
    abs_i32, call_rng, cat_attack_dispatch, get_entity_state, get_explosion_anchor, get_explosion_chance, get_explosion_span, get_metal_killer_pct,
    get_mini_surge, get_pos_x, get_soulstrike, get_surge_anchor, get_surge_chance, get_surge_level, get_surge_span, get_wave_chance, get_wave_level,
    get_wave_mini, has_attack_abilities, is_attack_long_range, roll_procs, AppContext, Entity, SurgeEvent, WaveRecord,
};

const SITE: &str = "cat_hit_executor";

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ExplosionEvent {
    pub faction: i32,
    pub slot: i32,
    pub frame: i32,
    pub x: i32,
    pub attack: i32,
    pub proc_flags: [u8; 12],
    pub metal_killer_pct: i32,
    pub hits: BTreeMap<i32, [u8; 5]>,
}

pub fn cat_hit_executor(ctx: &mut AppContext, slot: i32, targets: &[i32], attack: i32) -> Result<(), Fault> {
    roll_procs(ctx, 0, slot, attack)?;

    let crit = ctx.i32_at(AppContext::PROC_ROLLS)?;
    let p_knockback = ctx.i32_at(AppContext::PROC_ROLLS + 0x4)? != 0;
    let p_freeze = ctx.i32_at(AppContext::PROC_ROLLS + 0x8)? != 0;
    let p_slow = ctx.i32_at(AppContext::PROC_ROLLS + 0xc)? != 0;
    let p_weaken = ctx.i32_at(AppContext::PROC_ROLLS + 0x10)? != 0;
    let savage = ctx.i32_at(AppContext::PROC_ROLLS + 0x14)? != 0;
    let p_warp = ctx.i32_at(AppContext::PROC_ROLLS + 0x18)? != 0;
    let barrier_broke = ctx.i32_at(AppContext::PROC_ROLLS + 0x1c)? != 0;
    let p_curse = ctx.i32_at(AppContext::PROC_ROLLS + 0x20)? != 0;
    let p_toxic = ctx.i32_at(AppContext::PROC_ROLLS + 0x24)? != 0;
    let shield_pierced = ctx.i32_at(AppContext::PROC_ROLLS + 0x28)?;
    let p_drain = ctx.i32_at(AppContext::PROC_ROLLS + 0x2c)? != 0;
    let metal_killer_pct = get_metal_killer_pct(ctx, 0, slot)?;
    let mut index = 0usize;

    while index < targets.len() {
        let target = *targets.get(index).ok_or(Fault::IndexOutOfRange { site: SITE, index: index as i64, limit: targets.len() as i64 })?;

        if cat_attack_dispatch(
            ctx,
            0,
            slot,
            target,
            attack,
            0,
            (crit != 0) as u8,
            savage as u8,
            p_knockback as u8,
            p_freeze as u8,
            p_slow as u8,
            p_weaken as u8,
            p_warp as u8,
            p_curse as u8,
            barrier_broke as u8,
            (shield_pierced != 0) as u8,
            metal_killer_pct,
            100,
        )? {
            if get_soulstrike(ctx, 0, slot)? && get_entity_state(ctx, 1, target)? == 0xe {
                ctx.set_i32_at(AppContext::entity_field(1, target, Entity::HIT_SPARK_TYPE), 2)?;
            } else if is_attack_long_range(ctx, 0, slot, attack)? {
                ctx.set_i32_at(AppContext::entity_field(1, target, Entity::HIT_SPARK_TYPE), 1)?;
            } else {
                ctx.set_i32_at(AppContext::entity_field(1, target, Entity::HIT_SPARK_TYPE), 0)?;
            }
        }

        index += 1;
    }

    if !has_attack_abilities(ctx, 0, slot, attack)? {
        return Ok(());
    }

    let proc_flags = [
        (crit != 0) as u8,
        p_knockback as u8,
        p_freeze as u8,
        p_slow as u8,
        p_weaken as u8,
        p_warp as u8,
        p_curse as u8,
        barrier_broke as u8,
        savage as u8,
        p_toxic as u8,
        (shield_pierced != 0) as u8,
        p_drain as u8,
    ];

    if get_wave_chance(ctx, 0, slot)? > 0 {
        let roll = call_rng(ctx, 100);

        get_wave_mini(ctx, 0, slot)?;
        get_wave_chance(ctx, 0, slot)?;
        get_wave_chance(ctx, 0, slot)?;

        if get_wave_chance(ctx, 0, slot)? > roll {
            let mut pair = 0usize;

            loop {
                let first = AppContext::WAVE_RECORDS.wrapping_add(pair.wrapping_mul(AppContext::WAVE_RECORD_STRIDE * 2));
                let second = first.wrapping_add(AppContext::WAVE_RECORD_STRIDE);
                let free;
                let wave_index;

                if ctx.i32_at(first.wrapping_add(WaveRecord::KIND))? == 0 && ctx.i32_at(first.wrapping_add(WaveRecord::IN_USE))? == 0 {
                    free = first;
                    wave_index = pair.wrapping_mul(2);
                } else if ctx.i32_at(second.wrapping_add(WaveRecord::KIND))? == 0 && ctx.i32_at(second.wrapping_add(WaveRecord::IN_USE))? == 0 {
                    free = second;
                    wave_index = pair.wrapping_mul(2).wrapping_add(1);
                } else {
                    pair += 1;

                    if pair == 100 {
                        break;
                    }

                    continue;
                }

                let mini = get_wave_mini(ctx, 0, slot)? == 1;

                ctx.set_block_at::<1>(free.wrapping_add(WaveRecord::MINI), [mini as u8])?;
                ctx.set_i32_at(free.wrapping_add(WaveRecord::KIND), 1)?;
                ctx.set_i32_at(free.wrapping_add(WaveRecord::OWNER_SLOT), slot)?;
                ctx.set_i32_at(free.wrapping_add(WaveRecord::IN_USE), 1)?;
                ctx.set_i32_at(free.wrapping_add(WaveRecord::FRAME), 0)?;

                let x = get_pos_x(ctx, 0, slot)?;

                ctx.set_i32_at(free.wrapping_add(WaveRecord::POS_X), x)?;

                let level = get_wave_level(ctx, 0, slot)?;

                ctx.set_i32_at(free.wrapping_add(WaveRecord::LEVEL), level)?;
                ctx.set_i32_at(free.wrapping_add(WaveRecord::ATTACK), attack)?;
                ctx.set_block_at::<12>(free.wrapping_add(WaveRecord::PROC_FLAGS), proc_flags)?;
                ctx.set_i32_at(free.wrapping_add(WaveRecord::METAL_KILLER_PCT), metal_killer_pct)?;

                let mut victim = 0usize;

                while victim != 51 {
                    ctx.set_block_at::<1>(AppContext::WAVE_HITS.wrapping_add(victim.wrapping_mul(200)).wrapping_add(wave_index), [0])?;
                    victim += 1;
                }

                break;
            }
        }
    }

    if get_surge_chance(ctx, 0, slot)? > 0 {
        let roll = call_rng(ctx, 100);

        get_surge_chance(ctx, 0, slot)?;
        get_surge_chance(ctx, 0, slot)?;

        if get_surge_chance(ctx, 0, slot)? > roll {
            ctx.surge_events.push(SurgeEvent::default());

            let x = get_pos_x(ctx, 0, slot)?;
            let anchor = get_surge_anchor(ctx, 0, slot)?;
            let reach = call_rng(ctx, abs_i32(get_surge_span(ctx, 0, slot)?));
            let offset = if get_surge_span(ctx, 0, slot)? > 0 { reach.wrapping_neg() } else { reach };
            let level = get_surge_level(ctx, 0, slot)?;
            let mini = get_mini_surge(ctx, 0, slot)? != 0;
            let event = ctx.surge_events.last_mut().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?;

            event.faction = 0;
            event.slot = slot;
            event.frame = 0;
            event.x = x.wrapping_sub(anchor).wrapping_add(offset);
            event.level = level;
            event.attack = attack;
            event.proc_flags = proc_flags;
            event.metal_killer_pct = metal_killer_pct;
            event.mini = mini;
            event.kind = 0;
        }
    }

    if get_explosion_chance(ctx, 0, slot)? > 0 {
        let roll = call_rng(ctx, 100);

        get_explosion_chance(ctx, 0, slot)?;
        get_explosion_chance(ctx, 0, slot)?;

        if get_explosion_chance(ctx, 0, slot)? > roll {
            ctx.explosion_events.push(ExplosionEvent::default());

            let x = get_pos_x(ctx, 0, slot)?.wrapping_sub(get_explosion_anchor(ctx, 0, slot)?);
            let reach = call_rng(ctx, abs_i32(get_explosion_span(ctx, 0, slot)?));
            let offset = if get_explosion_span(ctx, 0, slot)? > 0 { reach.wrapping_neg() } else { reach };
            let event = ctx.explosion_events.last_mut().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?;

            event.faction = 0;
            event.slot = slot;
            event.frame = 0;
            event.x = offset.wrapping_add(x);
            event.attack = attack;
            event.proc_flags = proc_flags;
            event.metal_killer_pct = metal_killer_pct;
        }
    }

    Ok(())
}
