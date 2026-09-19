use crate::Fault;

use super::{AppContext, CatStats, EnemyStats, read_flag};

pub fn stat_soul_animation_type(
    ctx: &AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
) -> Result<i32, Fault> {
    let cell = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        AppContext::cat_stat(unit_id, form, CatStats::SOUL_ANIMATION_TYPE)
    } else {
        AppContext::enemy_stat(unit_id, EnemyStats::SOUL_ANIMATION_TYPE)
    };

    ctx.i32_at(cell)
}
