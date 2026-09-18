use crate::Fault;

use super::{read_flag, talent_targets_trait, AppContext, CatStats, EnemyStats};

pub fn trait_zombie(ctx: &mut AppContext, faction: i32, unit_id: i32, form: i32, allow_talent: u8) -> Result<bool, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_ZOMBIE))? != 0);
    }

    if ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::TARGET_ZOMBIE))? != 0 {
        return Ok(true);
    }

    if allow_talent == 0 {
        return Ok(false);
    }

    talent_targets_trait(ctx, faction, unit_id, form, 0x40)
}
