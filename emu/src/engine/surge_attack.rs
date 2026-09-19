use crate::Fault;

use super::{
    cat_attack_dispatch, enemy_attack_dispatch, get_counter_surge, get_counter_surge_once, get_counter_surge_pct, get_death_surge_anchor,
    get_death_surge_level, get_death_surge_mini, get_death_surge_pct, get_death_surge_span, get_mini_surge, get_pos_x, get_surge_anchor, get_surge_level,
    get_surge_span, set_counter_surge, set_counter_surge_once, AppContext, CounterSurgeEvent,
};

pub fn surge_attack(ctx: &mut AppContext, event_index: i32, target: i32, attack: i32) -> Result<(), Fault> {
    const SITE: &str = "surge_attack";

    let index = event_index as i64 as usize;
    let missing = Fault::IndexOutOfRange { site: SITE, index: event_index as i64, limit: ctx.surge_events.len() as i64 };
    let event = ctx.surge_events.get(index).ok_or(missing.clone())?;
    let faction = event.faction;
    let slot = event.slot;
    let hit = if faction != 0 {
        let mode = (event.mini as i32).wrapping_add(event.mini as i32);
        let proc_flags = event.proc_flags;

        enemy_attack_dispatch(
            ctx,
            2,
            slot,
            target,
            attack,
            mode,
            proc_flags[0],
            proc_flags[8],
            proc_flags[1],
            proc_flags[2],
            proc_flags[3],
            proc_flags[4],
            proc_flags[5],
            proc_flags[6],
            proc_flags[7],
            proc_flags[9],
            proc_flags[10],
            proc_flags[11],
            100,
        )?
    } else {
        let dmg_scale = match event.kind {
            1 => get_counter_surge_pct(ctx, 0, slot)?,
            2 => get_death_surge_pct(ctx, 0, slot)?,
            _ => 100,
        };
        let event = ctx.surge_events.get(index).ok_or(missing.clone())?;
        let mode = (event.mini as i32).wrapping_add(event.mini as i32);
        let proc_flags = event.proc_flags;
        let metal_killer_pct = event.metal_killer_pct;

        cat_attack_dispatch(
            ctx,
            2,
            slot,
            target,
            attack,
            mode,
            proc_flags[0],
            proc_flags[8],
            proc_flags[1],
            proc_flags[2],
            proc_flags[3],
            proc_flags[4],
            proc_flags[5],
            proc_flags[6],
            proc_flags[7],
            proc_flags[10],
            metal_killer_pct,
            dmg_scale,
        )?
    };

    if !hit {
        return Ok(());
    }

    let other = 1i32.wrapping_sub(faction);

    if !get_counter_surge(ctx, other, target)? {
        return Ok(());
    }

    let event = ctx.surge_events.get(index).ok_or(missing.clone())?;

    if event.hit_ticks.contains_key(&target) {
        return Ok(());
    }

    if event.kind == 1 {
        return Ok(());
    }

    ctx.counter_surge_events.push(CounterSurgeEvent::default());

    let x = get_pos_x(ctx, other, target)?;
    let kind = ctx.surge_events.get(index).ok_or(missing)?.kind;
    let level;
    let anchor;
    let span;
    let mini;

    if kind == 2 {
        level = get_death_surge_level(ctx, faction, slot)?;
        anchor = get_death_surge_anchor(ctx, faction, slot)?;
        span = get_death_surge_span(ctx, faction, slot)?;
        mini = get_death_surge_mini(ctx, faction, slot)?;
    } else {
        level = get_surge_level(ctx, faction, slot)?;
        anchor = get_surge_anchor(ctx, faction, slot)?;
        span = get_surge_span(ctx, faction, slot)?;
        mini = get_mini_surge(ctx, faction, slot)?;
    }

    let counter = ctx.counter_surge_events.last_mut().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?;

    counter.unknown_3 = 0;
    counter.faction = other;
    counter.slot = target;
    counter.x = x;
    counter.level = level;
    counter.anchor = anchor;
    counter.span = span;
    counter.mini = mini != 0;
    counter.unknown_2 = 0;

    if get_counter_surge_once(ctx, other, target)? {
        set_counter_surge(ctx, other, target, 0)?;
        set_counter_surge_once(ctx, other, target, 0)?;
    }

    Ok(())
}
