use crate::{operation, Fault};

use super::{
    call_rng, get_battle_status, get_castle_enemy_row, get_entity_base_idx, get_hp, get_total_damage_taken, has_castle_enemy,
    read_flag, spawn_entity, stage_entry_base_trigger, stage_entry_count, stage_entry_death_gate, stage_entry_enemy_id,
    stage_entry_keep_delay, stage_entry_respawn_max, stage_entry_respawn_min, stage_entry_row, stage_entry_z_max,
    stage_entry_z_min, AppContext, EnemyStats, Entity, ENEMY_STATS, ENEMY_STATS_STRIDE,
};

const SITE: &str = "enemy_schedule_tick";

pub fn enemy_schedule_tick(ctx: &mut AppContext) -> Result<(), Fault> {
    let kills = ctx.i32_at(AppContext::KILLS_SINCE_SPAWN_TICK)?;

    ctx.set_i32_at(AppContext::KILLS_SINCE_SPAWN_TICK, 0)?;

    if get_battle_status(ctx)? != 0 {
        return Ok(());
    }

    if read_flag(ctx, AppContext::faction_flags(1))? & 1 != 0 {
        return Ok(());
    }

    let countdown = ctx.i32_at(AppContext::SPAWN_COUNTDOWN)?.wrapping_sub(1);

    ctx.set_i32_at(AppContext::SPAWN_COUNTDOWN, countdown)?;

    let mut row = 0usize;

    while row < ctx.stage_enemies.len() {
        let mut triggered = false;
        let enemy_row = ctx.stage_enemies[row];
        let mut met = false;

        if has_castle_enemy(ctx)?
            && ctx.i32_at(((get_castle_enemy_row(ctx)? as i64) * ENEMY_STATS_STRIDE as i64 + (ENEMY_STATS + EnemyStats::TRAIT_DOJO) as i64) as usize)? != 0
        {
            let base_idx = get_entity_base_idx(ctx)?;

            if get_total_damage_taken(ctx, 1, base_idx)? >= stage_entry_base_trigger(&enemy_row) {
                met = true;
            }
        } else {
            let hp = get_hp(ctx, 1, 0)?;
            let max_hp = ctx.i32_at(AppContext::entity_field(1, 0, Entity::MAX_HP))?;

            if hp <= operation::div_100(stage_entry_base_trigger(&enemy_row).wrapping_mul(max_hp) as i64) as i32 {
                met = true;
            }
        }

        let state = ctx.spawn_states.get_mut(row).ok_or(Fault::IndexOutOfRange { site: SITE, index: row as i64, limit: 0 })?;

        if met {
            state[2] = state[2].wrapping_add(kills);
            triggered = true;
        }

        if state[2] < stage_entry_death_gate(&enemy_row) && state[0] <= 1 {
            state[0] = 1;
        } else if triggered {
            if stage_entry_count(&enemy_row) == 0 || state[1] < stage_entry_count(&enemy_row) {
                state[0] = state[0].wrapping_sub(1);
            }
        } else if stage_entry_keep_delay(&enemy_row) == 0 {
            state[0] = 1;
        }

        row += 1;
    }

    if ctx.i32_at(AppContext::SPAWN_COUNTDOWN)? > 0 {
        return Ok(());
    }

    let mut alive = 0i32;

    for slot in 1..0x33 {
        alive = alive.wrapping_add((ctx.i32_at(AppContext::entity_field(1, slot, Entity::OCCUPANT))? != 0) as i32);
    }

    if alive >= ctx.i32_at(AppContext::STAGE_MAX_ENEMIES)? {
        return Ok(());
    }

    let mut row = ctx.stage_enemies.len() as i32;

    loop {
        if row <= 0 {
            return Ok(());
        }

        row -= 1;

        let waiting = ctx.spawn_states.get(row as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: row as i64, limit: 0 })?[0];

        if waiting <= 0 {
            break;
        }
    }

    let enemy_row = ctx.stage_enemies[row as usize];

    if spawn_entity(ctx, 1, stage_entry_row(&enemy_row), 0, stage_entry_z_min(&enemy_row), stage_entry_z_max(&enemy_row), 0, row)? < 0 {
        return Ok(());
    }

    let spawn_min = ctx.i32_at(AppContext::STAGE_SPAWN_MIN)?;
    let spawn_span = ctx.i32_at(AppContext::STAGE_SPAWN_MAX)?.wrapping_sub(spawn_min).wrapping_add(1);
    let next_spawn = call_rng(ctx, spawn_span).wrapping_add(ctx.i32_at(AppContext::STAGE_SPAWN_MIN)?);

    ctx.set_i32_at(AppContext::SPAWN_COUNTDOWN, next_spawn)?;

    let respawn_span = stage_entry_respawn_max(&enemy_row).wrapping_sub(stage_entry_respawn_min(&enemy_row)).wrapping_add(1);
    let respawn = stage_entry_respawn_min(&enemy_row).wrapping_add(call_rng(ctx, respawn_span));
    let state = ctx.spawn_states.get_mut(row as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: row as i64, limit: 0 })?;

    state[0] = respawn;
    state[1] = state[1].wrapping_add(1);

    let seen = ((stage_entry_enemy_id(&enemy_row) as i64) * 4 + AppContext::SEEN_ENEMIES as i64) as usize;

    if ctx.i32_at(seen)? == 0 {
        ctx.set_i32_at(seen, 1)?;
    }

    Ok(())
}
