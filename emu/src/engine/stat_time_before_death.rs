use crate::Fault;

use super::{read_flag, AppContext, CatStats, EnemyStats};

pub fn stat_time_before_death(ctx: &AppContext, faction: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    let cell = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        AppContext::cat_stat(unit_id, form, CatStats::TIME_BEFORE_DEATH)
    } else {
        AppContext::enemy_stat(unit_id, EnemyStats::TIME_BEFORE_DEATH)
    };

    ctx.i32_at(cell)
}
