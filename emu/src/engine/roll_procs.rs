use crate::Fault;

use super::{
    AppContext, call_rng, get_barrier_breaker_chance, get_critical_chance, get_curse_chance,
    get_drain_chance, get_freeze_chance, get_knockback_chance, get_savage_blow_chance,
    get_shield_pierce_chance, get_slow_chance, get_toxic_chance, get_warp_chance,
    get_weaken_chance, has_attack_abilities, has_barrier_breaker_chance, has_critical_chance,
    has_curse_chance, has_drain_chance, has_freeze_chance, has_knockback_chance,
    has_savage_blow_chance, has_shield_pierce_chance, has_slow_chance, has_toxic_chance,
    has_warp_chance, has_weaken_chance,
};

pub fn roll_procs(ctx: &mut AppContext, faction: i32, slot: i32, attack: i32) -> Result<(), Fault> {
    ctx.zero(AppContext::PROC_ROLLS, 0x30)?;

    if !has_attack_abilities(ctx, faction, slot, attack)? {
        return Ok(());
    }

    let roll = call_rng(ctx, 0x64);
    let hit = (get_critical_chance(ctx, faction, slot)? > roll) as i32;

    ctx.set_i32_at(AppContext::PROC_ROLLS, hit)?;

    if has_critical_chance(ctx, faction, slot)? {
        get_critical_chance(ctx, faction, slot)?;
        get_critical_chance(ctx, faction, slot)?;
    }

    let roll = call_rng(ctx, 0x64);
    let hit = (get_knockback_chance(ctx, faction, slot)? > roll) as i32;

    ctx.set_i32_at(AppContext::PROC_ROLLS + 0x4, hit)?;

    if has_knockback_chance(ctx, faction, slot)? {
        get_knockback_chance(ctx, faction, slot)?;
        get_knockback_chance(ctx, faction, slot)?;
    }

    let roll = call_rng(ctx, 0x64);
    let hit = (get_freeze_chance(ctx, faction, slot)? > roll) as i32;

    ctx.set_i32_at(AppContext::PROC_ROLLS + 0x8, hit)?;

    if has_freeze_chance(ctx, faction, slot)? {
        get_freeze_chance(ctx, faction, slot)?;
        get_freeze_chance(ctx, faction, slot)?;
    }

    let roll = call_rng(ctx, 0x64);
    let hit = (get_slow_chance(ctx, faction, slot)? > roll) as i32;

    ctx.set_i32_at(AppContext::PROC_ROLLS + 0xc, hit)?;

    if has_slow_chance(ctx, faction, slot)? {
        get_slow_chance(ctx, faction, slot)?;
        get_slow_chance(ctx, faction, slot)?;
    }

    let roll = call_rng(ctx, 0x64);
    let hit = (get_weaken_chance(ctx, faction, slot)? > roll) as i32;

    ctx.set_i32_at(AppContext::PROC_ROLLS + 0x10, hit)?;

    if has_weaken_chance(ctx, faction, slot)? {
        get_weaken_chance(ctx, faction, slot)?;
        get_weaken_chance(ctx, faction, slot)?;
    }

    let roll = call_rng(ctx, 0x64);
    let hit = (get_savage_blow_chance(ctx, faction, slot)? > roll) as i32;

    ctx.set_i32_at(AppContext::PROC_ROLLS + 0x14, hit)?;

    if has_savage_blow_chance(ctx, faction, slot)? {
        get_savage_blow_chance(ctx, faction, slot)?;
        get_savage_blow_chance(ctx, faction, slot)?;
    }

    let roll = call_rng(ctx, 0x64);
    let hit = (get_warp_chance(ctx, faction, slot)? > roll) as i32;

    ctx.set_i32_at(AppContext::PROC_ROLLS + 0x18, hit)?;

    if has_warp_chance(ctx, faction, slot)? {
        get_warp_chance(ctx, faction, slot)?;
        get_warp_chance(ctx, faction, slot)?;
    }

    let roll = call_rng(ctx, 0x64);
    let hit = (get_barrier_breaker_chance(ctx, faction, slot)? > roll) as i32;

    ctx.set_i32_at(AppContext::PROC_ROLLS + 0x1c, hit)?;

    if has_barrier_breaker_chance(ctx, faction, slot)? {
        get_barrier_breaker_chance(ctx, faction, slot)?;
        get_barrier_breaker_chance(ctx, faction, slot)?;
    }

    let roll = call_rng(ctx, 0x64);
    let hit = (get_curse_chance(ctx, faction, slot)? > roll) as i32;

    ctx.set_i32_at(AppContext::PROC_ROLLS + 0x20, hit)?;

    if has_curse_chance(ctx, faction, slot)? {
        get_curse_chance(ctx, faction, slot)?;
        get_curse_chance(ctx, faction, slot)?;
    }

    let roll = call_rng(ctx, 0x64);
    let hit = (get_toxic_chance(ctx, faction, slot)? > roll) as i32;

    ctx.set_i32_at(AppContext::PROC_ROLLS + 0x24, hit)?;

    if has_toxic_chance(ctx, faction, slot)? {
        get_toxic_chance(ctx, faction, slot)?;
        get_toxic_chance(ctx, faction, slot)?;
    }

    let roll = call_rng(ctx, 0x64);
    let hit = (get_shield_pierce_chance(ctx, faction, slot)? > roll) as i32;

    ctx.set_i32_at(AppContext::PROC_ROLLS + 0x28, hit)?;

    if has_shield_pierce_chance(ctx, faction, slot)? {
        get_shield_pierce_chance(ctx, faction, slot)?;
        get_shield_pierce_chance(ctx, faction, slot)?;
    }

    let roll = call_rng(ctx, 0x64);
    let hit = (get_drain_chance(ctx, faction, slot)? > roll) as i32;

    ctx.set_i32_at(AppContext::PROC_ROLLS + 0x2c, hit)?;

    if has_drain_chance(ctx, faction, slot)? {
        get_drain_chance(ctx, faction, slot)?;
        get_drain_chance(ctx, faction, slot)?;
    }

    Ok(())
}
