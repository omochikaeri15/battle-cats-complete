use crate::Fault;

use super::{read_flag, talent_targets_trait, AppContext, CatStats, EnemyStats};

pub fn stat_has_trait(ctx: &mut AppContext, faction: i32, unit_id: i32, form: i32, trait_index: i32) -> Result<bool, Fault> {
    match trait_index {
        0x00 => {
            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_RED))? != 0);
            }

            if ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::TARGET_RED))? != 0 {
                return Ok(true);
            }

            talent_targets_trait(ctx, faction, unit_id, form, 0x1)
        }
        0x01 => {
            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_FLOATING))? != 0);
            }

            if ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::TARGET_FLOATING))? != 0 {
                return Ok(true);
            }

            talent_targets_trait(ctx, faction, unit_id, form, 0x2)
        }
        0x02 => {
            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_DARK))? != 0);
            }

            if ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::TARGET_DARK))? != 0 {
                return Ok(true);
            }

            talent_targets_trait(ctx, faction, unit_id, form, 0x4)
        }
        0x03 => {
            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_METAL))? != 0);
            }

            if ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::TARGET_METAL))? != 0 {
                return Ok(true);
            }

            talent_targets_trait(ctx, faction, unit_id, form, 0x8)
        }
        0x04 => {
            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_ANGEL))? != 0);
            }

            if ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::TARGET_ANGEL))? != 0 {
                return Ok(true);
            }

            talent_targets_trait(ctx, faction, unit_id, form, 0x10)
        }
        0x05 => {
            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_ALIEN))? != 0);
            }

            if ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::TARGET_ALIEN))? != 0 {
                return Ok(true);
            }

            talent_targets_trait(ctx, faction, unit_id, form, 0x20)
        }
        0x06 => {
            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_ZOMBIE))? != 0);
            }

            if ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::TARGET_ZOMBIE))? != 0 {
                return Ok(true);
            }

            talent_targets_trait(ctx, faction, unit_id, form, 0x40)
        }
        0x07 => {
            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_RELIC))? != 0);
            }

            if ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::TARGET_RELIC))? != 0 {
                return Ok(true);
            }

            talent_targets_trait(ctx, faction, unit_id, form, 0x80)
        }
        0x08 => {
            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_AKU))? != 0);
            }

            if ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::TARGET_AKU))? != 0 {
                return Ok(true);
            }

            talent_targets_trait(ctx, faction, unit_id, form, 0x800)
        }
        0x09 => {
            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_TRAITLESS))? != 0);
            }

            if ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::TARGET_TRAITLESS))? != 0 {
                return Ok(true);
            }

            talent_targets_trait(ctx, faction, unit_id, form, 0x100)
        }
        0x0a => {
            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_COLOSSUS))? != 0);
            }

            Ok(false)
        }
        0x0b => {
            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_BEHEMOTH))? != 0);
            }

            Ok(false)
        }
        0x0c => {
            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_SAGE))? != 0);
            }

            Ok(false)
        }
        0x0d => {
            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_WITCH))? != 0);
            }

            if ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::TARGET_WITCH))? != 0 {
                return Ok(true);
            }

            talent_targets_trait(ctx, faction, unit_id, form, 0x200)
        }
        0x0e => {
            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_EVA))? != 0);
            }

            if ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::TARGET_EVA))? != 0 {
                return Ok(true);
            }

            talent_targets_trait(ctx, faction, unit_id, form, 0x400)
        }
        0x0f => {
            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_KAIJIN))? != 0);
            }

            Ok(false)
        }
        _ => Ok(false),
    }
}
