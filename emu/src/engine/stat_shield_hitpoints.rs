use crate::{operation, Fault};

use super::{
    get_global_map_id, get_star_level, get_star_multiplier, read_flag, stage_entry_magnification, AppContext,
    EOC_CHAPTER_HP_MUL, EnemyStats,
};

pub fn stat_shield_hitpoints(ctx: &mut AppContext, faction: i32, unit_id: i32, _form: i32, enemy_row: i32) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        return Ok(0);
    }

    if ctx.i32_at(AppContext::CHAPTER_MODE)? > 2 || ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0 {
        let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
        let shield = ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::SHIELD_HITPOINTS))?;
        let row = ctx.stage_enemies.get(enemy_row as usize).ok_or(Fault::IndexOutOfRange {
            site: "stat_shield_hitpoints",
            index: enemy_row as i64,
            limit: ctx.stage_enemies.len() as i64,
        })?;
        let scaled = operation::div_100(stage_entry_magnification(row).wrapping_mul(shield).wrapping_add(0x32));

        if chapter != 3 {
            return Ok(scaled);
        }

        let map_id = get_global_map_id(ctx, 0)?;
        let star = get_star_level(ctx)?;

        return Ok(operation::div_100(get_star_multiplier(&ctx.star_multipliers, map_id, star)?.wrapping_mul(scaled)));
    }

    let shield = ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::SHIELD_HITPOINTS))?;
    let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
    let bonus = *EOC_CHAPTER_HP_MUL.get(chapter as usize).ok_or(Fault::IndexOutOfRange {
        site: "stat_shield_hitpoints",
        index: chapter as i64,
        limit: 3,
    })?;

    Ok(operation::div_10(bonus.wrapping_add(0xa).wrapping_mul(shield).wrapping_add(5)))
}
