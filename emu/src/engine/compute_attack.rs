use crate::{operation, Fault};

use super::{
    ex_redirect_check_a, ex_redirect_check_b, get_cat_combo_bonus, get_global_map_id, get_map_type,
    get_orb_value_max, get_star_level, get_star_multiplier, get_talent_value, get_treasure_uncapped,
    get_treasure_value, is_ex_map_68, read_flag, stage_entry_atk_mag, AppContext, CatStats, EOC_CHAPTER_HP_MUL,
    EnemyStats, STAT_ATTACK_COLUMNS,
};

#[allow(clippy::too_many_arguments)]
pub fn compute_attack(
    ctx: &mut AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
    level: i32,
    mag_slot: i32,
    atk_idx: i32,
    skip_mods: u8,
    skip_mult: u8,
) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        let leveled = if level <= 0 {
            0i64
        } else {
            let column = *STAT_ATTACK_COLUMNS.get(atk_idx as usize).ok_or(Fault::IndexOutOfRange {
                site: "compute_attack",
                index: atk_idx as i64,
                limit: 3,
            })? as i64;
            let mut scaled = (ctx.i32_at(AppContext::cat_stat(unit_id, form, (column * 4) as usize))? as i64).wrapping_mul(0x64);

            if level != 1 {
                let mut step = 1i32;

                loop {
                    if step as u32 <= 0xc7 {
                        let column = *STAT_ATTACK_COLUMNS.get(atk_idx as usize).ok_or(Fault::IndexOutOfRange {
                            site: "compute_attack",
                            index: atk_idx as i64,
                            limit: 3,
                        })? as i64;
                        let base = ctx.i32_at(AppContext::cat_stat(unit_id, form, (column * 4) as usize))? as i64;
                        let curve = (unit_id as i64) * 0x50 + ((step as u8 / 10) as i64) * 4 + AppContext::UNIT_LEVEL_CURVE as i64;

                        scaled = scaled.wrapping_add((ctx.i32_at(curve as usize)? as i64).wrapping_mul(base));
                    }

                    step = step.wrapping_add(1);

                    if step == level {
                        break;
                    }
                }
            }

            operation::div_100(scaled.wrapping_add(0x32))
        };

        if skip_mods != 0 {
            return Ok(leveled as i32);
        }

        let treasure = get_treasure_value(ctx, &ctx.treasure_store, 0xa)? as i64;
        let with_treasure = operation::div_100(treasure.wrapping_mul(leveled)).wrapping_add(leveled);

        let combo = get_cat_combo_bonus(ctx, &ctx.combo_store, 0, unit_id)?.wrapping_add(0x64) as i64;
        let mut hp = operation::div_100(combo.wrapping_mul(with_treasure));

        let mut boost = get_talent_value(ctx, faction, unit_id, form, 0x1f, 0)?;

        if form > 1 {
            let orb_abil = if get_map_type(ctx, 0)? == 0 || ex_redirect_check_a(ctx)? {
                Some(9)
            } else if get_map_type(ctx, 0)? == -9 || ex_redirect_check_b(ctx)? || is_ex_map_68(ctx)? {
                Some(0x10)
            } else {
                None
            };

            if let Some(abil) = orb_abil {
                boost = boost.wrapping_add(get_orb_value_max(ctx, unit_id, abil, 1, 0)?);
            }
        }

        if boost > 0 {
            hp = operation::div_100(hp.wrapping_mul(boost.wrapping_add(0x64) as u32 as i64));
        }

        return Ok(hp as i32);
    }

    let mut hp = if ctx.i32_at(AppContext::CHAPTER_MODE)? > 2 || ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0 {
        let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
        let column = *STAT_ATTACK_COLUMNS.get((3 + atk_idx as i64) as usize).ok_or(Fault::IndexOutOfRange {
            site: "compute_attack",
            index: atk_idx as i64,
            limit: 3,
        })? as i64;
        let base = ctx.i32_at(AppContext::enemy_stat(unit_id, (column * 4) as usize))? as i64;
        let entry = ctx.stage_enemies.get(mag_slot as usize).ok_or(Fault::IndexOutOfRange {
            site: "compute_attack",
            index: mag_slot as i64,
            limit: ctx.stage_enemies.len() as i64,
        })?;
        let mut scaled = operation::div_100((stage_entry_atk_mag(entry) as i64).wrapping_mul(base).wrapping_add(0x32));

        if chapter == 3 {
            let mut multiplier = 0x64i64;

            if skip_mult == 0 {
                let map_id = get_global_map_id(ctx, 0)?;
                let star = get_star_level(ctx)?;

                multiplier = get_star_multiplier(&ctx.star_multipliers, map_id, star)? as i64;
            }

            scaled = operation::div_100(multiplier.wrapping_mul(scaled));
        }

        scaled
    } else {
        let column = *STAT_ATTACK_COLUMNS.get((3 + atk_idx as i64) as usize).ok_or(Fault::IndexOutOfRange {
            site: "compute_attack",
            index: atk_idx as i64,
            limit: 3,
        })? as i64;
        let base = ctx.i32_at(AppContext::enemy_stat(unit_id, (column * 4) as usize))? as i64;
        let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
        let bonus = *EOC_CHAPTER_HP_MUL.get(chapter as usize).ok_or(Fault::IndexOutOfRange {
            site: "compute_attack",
            index: chapter as i64,
            limit: 3,
        })? as i64;

        operation::div_10(bonus.wrapping_add(0xa).wrapping_mul(base).wrapping_add(5))
    };

    let alien = if read_flag(ctx, AppContext::faction_flags(1))? & 1 != 0 {
        ctx.i32_at(AppContext::cat_stat(unit_id, 0, CatStats::TARGET_ALIEN))? != 0
    } else {
        ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_ALIEN))? != 0
    };

    let effect = if alien && (read_flag(ctx, AppContext::faction_flags(1))? & 1 != 0 || ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_STARRED_ALIEN))? == 0) {
        Some(0x10)
    } else {
        let alien = if read_flag(ctx, AppContext::faction_flags(1))? & 1 != 0 {
            ctx.i32_at(AppContext::cat_stat(unit_id, 0, CatStats::TARGET_ALIEN))? != 0
        } else {
            ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_ALIEN))? != 0
        };

        if alien && read_flag(ctx, AppContext::faction_flags(1))? & 1 == 0 && ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_STARRED_ALIEN))? == 1 {
            Some(0x12)
        } else if read_flag(ctx, AppContext::faction_flags(1))? & 1 == 0 && ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_STARRED_ALIEN))? == 2 {
            Some(0x16)
        } else if read_flag(ctx, AppContext::faction_flags(1))? & 1 == 0 && ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_STARRED_ALIEN))? == 3 {
            Some(0x17)
        } else if read_flag(ctx, AppContext::faction_flags(1))? & 1 == 0 && ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_STARRED_ALIEN))? == 4 {
            Some(0x18)
        } else {
            None
        }
    };

    if let Some(effect) = effect {
        let uncapped = get_treasure_uncapped(ctx, &ctx.treasure_store, effect)?;
        let gap = uncapped.wrapping_sub(get_treasure_value(ctx, &ctx.treasure_store, effect)?) as i64;

        hp = hp.wrapping_add(operation::div_100(gap.wrapping_mul(hp)));
    }

    Ok(hp as i32)
}
