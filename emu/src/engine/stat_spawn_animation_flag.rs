use crate::Fault;

use super::{read_flag, AppContext, CatStats, EnemyStats};

pub fn stat_spawn_animation_flag(ctx: &AppContext, faction: i32, unit_id: i32, form: i32) -> Result<bool, Fault> {
    let cell = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        AppContext::cat_stat(unit_id, form, CatStats::SPAWN_ANIMATION_FLAG)
    } else {
        AppContext::enemy_stat(unit_id, EnemyStats::SPAWN_ANIMATION_FLAG)
    };

    Ok(ctx.i32_at(cell)? != 0)
}
