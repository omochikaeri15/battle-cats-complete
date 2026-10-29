use crate::{Fault, ops};

use super::{
    AppContext, get_base_hp, get_base_max_hp, get_entity_state, is_boss, slot_occupied,
    stage_entry_base_trigger, stage_entry_count, stage_entry_is_boss,
};

pub fn is_boss_guarding_base(ctx: &AppContext) -> Result<bool, Fault> {
    if ctx.i32_at(AppContext::STAGE_BOSS_GUARD)? == 0 {
        return Ok(false);
    }

    let hp = get_base_hp(ctx, 1)?;
    let max_hp = get_base_max_hp(ctx, 1)?;
    let mut row = 0usize;

    while row < ctx.stage_enemies.len() {
        let enemy_row = ctx.stage_enemies.get(row).ok_or(Fault::index_out_of_range(row as i64, ctx.stage_enemies.len() as i64))?;

        if stage_entry_is_boss(enemy_row)
            && hp
                <= ops::div_100(
                    stage_entry_base_trigger(enemy_row).wrapping_mul(max_hp) as i64
                ) as i32
        {
            let spawned = ctx.spawn_states.get(row).ok_or(Fault::index_out_of_range(row as i64, ctx.spawn_states.len() as i64))?[1];

            if spawned < stage_entry_count(enemy_row) {
                return Ok(true);
            }
        }

        row += 1;
    }

    let mut slot = 0i32;

    loop {
        if slot_occupied(ctx, 1, slot)? != 0
            && is_boss(ctx, 1, slot)?
            && get_entity_state(ctx, 1, slot)? != 4
        {
            return Ok(true);
        }

        slot += 1;

        if slot == 0x33 {
            return Ok(false);
        }
    }
}
