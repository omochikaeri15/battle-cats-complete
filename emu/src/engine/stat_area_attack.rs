use crate::Fault;

use super::{AppContext, CatStats, EnemyStats, read_flag};

pub fn stat_area_attack(
    ctx: &AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
) -> Result<bool, Fault> {
    let cell = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        AppContext::cat_stat(unit_id, form, CatStats::AREA_ATTACK)
    } else {
        AppContext::enemy_stat(unit_id, EnemyStats::AREA_ATTACK)
    };

    Ok(ctx.i32_at(cell)? != 0)
}
