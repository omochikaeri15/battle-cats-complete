use crate::{Fault, operation};

use super::{
    AppContext, get_base_hp, get_base_max_hp, is_boss_guarding_base, stage_entry_base_trigger,
    stage_entry_count, stage_entry_is_boss,
};

pub fn base_resists_one_shot(ctx: &AppContext) -> Result<bool, Fault> {
    if ctx.i32_at(AppContext::STAGE_BOSS_GUARD)? == 0 {
        return Ok(false);
    }

    let hp = get_base_hp(ctx, 1)?;
    let max_hp = get_base_max_hp(ctx, 1)?;
    let mut row = 0usize;

    while row < ctx.stage_enemies.len() {
        let enemy_row = ctx.stage_enemies.get(row).ok_or(Fault::IndexOutOfRange {
            site: "base_resists_one_shot",
            index: row as i64,
            limit: ctx.stage_enemies.len() as i64,
        })?;

        if stage_entry_is_boss(enemy_row)
            && hp
                >= operation::div_100(
                    stage_entry_base_trigger(enemy_row).wrapping_mul(max_hp) as i64
                ) as i32
        {
            let spawned = ctx.spawn_states.get(row).ok_or(Fault::IndexOutOfRange {
                site: "base_resists_one_shot",
                index: row as i64,
                limit: ctx.spawn_states.len() as i64,
            })?[1];

            if spawned < stage_entry_count(enemy_row) {
                return Ok(true);
            }
        }

        row += 1;
    }

    is_boss_guarding_base(ctx)
}
