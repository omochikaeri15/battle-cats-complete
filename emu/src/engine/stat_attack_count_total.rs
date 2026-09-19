use crate::Fault;

use super::{AppContext, CatStats, EnemyStats, read_flag};

pub fn stat_attack_count_total(
    ctx: &AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
) -> Result<i32, Fault> {
    let cell = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        AppContext::cat_stat(unit_id, form, CatStats::ATTACK_COUNT_TOTAL)
    } else {
        AppContext::enemy_stat(unit_id, EnemyStats::ATTACK_COUNT_TOTAL)
    };

    ctx.i32_at(cell)
}
