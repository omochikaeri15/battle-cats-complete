use crate::Fault;

use super::{
    AppContext, Entity, ExplosionEvent, SurgeEvent, WaveRecord, abs_i32, call_rng,
    enemy_attack_dispatch, get_explosion_anchor, get_explosion_chance, get_explosion_span,
    get_mini_surge, get_pos_x, get_surge_anchor, get_surge_chance, get_surge_level, get_surge_span,
    get_wave_chance, get_wave_level, get_wave_mini, has_attack_abilities, is_attack_long_range,
    roll_procs,
};

const SITE: &str = "enemy_hit_executor";

pub fn enemy_hit_executor(
    ctx: &mut AppContext,
    slot: i32,
    targets: &[i32],
    attack: i32,
) -> Result<(), Fault> {
    roll_procs(ctx, 1, slot, attack)?;

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
    let mut index = 0usize;

    while index < targets.len() {
        let target = *targets.get(index).ok_or(Fault::IndexOutOfRange {
            site: SITE,
            index: index as i64,
            limit: targets.len() as i64,
        })?;

        if enemy_attack_dispatch(
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
            p_toxic as u8,
            (shield_pierced != 0) as u8,
            p_drain as u8,
            100,
        )? {
            let long_range = is_attack_long_range(ctx, 1, slot, attack)?;

            ctx.set_i32_at(
                AppContext::entity_field(0, target, Entity::HIT_SPARK_TYPE),
                long_range as i32,
            )?;
        }

        index += 1;
    }

    if !has_attack_abilities(ctx, 1, slot, attack)? {
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

    if get_wave_chance(ctx, 1, slot)? > 0 {
        let roll = call_rng(ctx, 100);

        get_wave_mini(ctx, 1, slot)?;
        get_wave_chance(ctx, 1, slot)?;
        get_wave_chance(ctx, 1, slot)?;

        if get_wave_chance(ctx, 1, slot)? > roll {
            let mut pair = 0usize;

            loop {
                let first = AppContext::WAVE_RECORDS
                    .wrapping_add(pair.wrapping_mul(AppContext::WAVE_RECORD_STRIDE * 2));
                let second = first.wrapping_add(AppContext::WAVE_RECORD_STRIDE);
                let free;
                let wave_index;

                if ctx.i32_at(first.wrapping_add(WaveRecord::KIND))? == 0
                    && ctx.i32_at(first.wrapping_add(WaveRecord::IN_USE))? == 0
                {
                    free = first;
                    wave_index = pair.wrapping_mul(2);
                } else if ctx.i32_at(second.wrapping_add(WaveRecord::KIND))? == 0
                    && ctx.i32_at(second.wrapping_add(WaveRecord::IN_USE))? == 0
                {
                    free = second;
                    wave_index = pair.wrapping_mul(2).wrapping_add(1);
                } else {
                    pair += 1;

                    if pair == 100 {
                        break;
                    }

                    continue;
                }

                let mini = get_wave_mini(ctx, 1, slot)? == 1;

                ctx.set_block_at::<1>(free.wrapping_add(WaveRecord::MINI), [mini as u8])?;
                ctx.set_i32_at(free.wrapping_add(WaveRecord::KIND), 2)?;
                ctx.set_i32_at(free.wrapping_add(WaveRecord::OWNER_SLOT), slot)?;
                ctx.set_i32_at(free.wrapping_add(WaveRecord::IN_USE), 2)?;
                ctx.set_i32_at(free.wrapping_add(WaveRecord::FRAME), 0)?;

                let x = get_pos_x(ctx, 1, slot)?;

                ctx.set_i32_at(free.wrapping_add(WaveRecord::POS_X), x)?;

                let level = get_wave_level(ctx, 1, slot)?;

                ctx.set_i32_at(free.wrapping_add(WaveRecord::LEVEL), level)?;
                ctx.set_i32_at(free.wrapping_add(WaveRecord::ATTACK), attack)?;
                ctx.set_block_at::<12>(free.wrapping_add(WaveRecord::PROC_FLAGS), proc_flags)?;
                ctx.set_i32_at(free.wrapping_add(WaveRecord::METAL_KILLER_PCT), 0)?;

                let mut victim = 0usize;

                while victim != 51 {
                    ctx.set_block_at::<1>(
                        AppContext::WAVE_HITS
                            .wrapping_add(victim.wrapping_mul(200))
                            .wrapping_add(wave_index),
                        [0],
                    )?;
                    victim += 1;
                }

                break;
            }
        }
    }

    if get_surge_chance(ctx, 1, slot)? > 0 {
        let roll = call_rng(ctx, 100);

        get_surge_chance(ctx, 1, slot)?;
        get_surge_chance(ctx, 1, slot)?;

        if get_surge_chance(ctx, 1, slot)? > roll {
            ctx.surge_events.push(SurgeEvent::default());

            let x = get_pos_x(ctx, 1, slot)?;
            let anchor = get_surge_anchor(ctx, 1, slot)?;
            let reach = call_rng(ctx, abs_i32(get_surge_span(ctx, 1, slot)?));
            let offset = if get_surge_span(ctx, 1, slot)? > 0 {
                reach
            } else {
                reach.wrapping_neg()
            };
            let level = get_surge_level(ctx, 1, slot)?;
            let mini = get_mini_surge(ctx, 1, slot)? != 0;
            let event = ctx.surge_events.last_mut().ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: 0,
                limit: 0,
            })?;

            event.faction = 1;
            event.slot = slot;
            event.frame = 0;
            event.x = offset.wrapping_add(anchor.wrapping_add(x));
            event.level = level;
            event.attack = attack;
            event.proc_flags = proc_flags;
            event.metal_killer_pct = 0;
            event.mini = mini;
            event.kind = 0;
        }
    }

    if get_explosion_chance(ctx, 1, slot)? > 0 {
        let roll = call_rng(ctx, 100);

        get_explosion_chance(ctx, 1, slot)?;
        get_explosion_chance(ctx, 1, slot)?;

        if get_explosion_chance(ctx, 1, slot)? > roll {
            ctx.explosion_events.push(ExplosionEvent::default());

            let x = get_pos_x(ctx, 1, slot)?.wrapping_add(get_explosion_anchor(ctx, 1, slot)?);
            let reach = call_rng(ctx, abs_i32(get_explosion_span(ctx, 1, slot)?));
            let offset = if get_explosion_span(ctx, 1, slot)? > 0 {
                reach
            } else {
                reach.wrapping_neg()
            };
            let event = ctx
                .explosion_events
                .last_mut()
                .ok_or(Fault::IndexOutOfRange {
                    site: SITE,
                    index: 0,
                    limit: 0,
                })?;

            event.faction = 1;
            event.slot = slot;
            event.frame = 0;
            event.x = offset.wrapping_add(x);
            event.attack = attack;
            event.proc_flags = proc_flags;
            event.metal_killer_pct = 0;
        }
    }

    Ok(())
}
