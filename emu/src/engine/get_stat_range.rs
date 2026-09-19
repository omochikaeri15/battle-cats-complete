use crate::Fault;

use super::{AppContext, CatStats, EnemyStats, read_flag};

pub fn get_stat_range(
    ctx: &AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
) -> Result<i32, Fault> {
    let row = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        AppContext::cat_stat(unit_id, form, CatStats::HITPOINTS)
    } else {
        AppContext::enemy_stat(unit_id, EnemyStats::HITPOINTS)
    };

    ctx.i32_at(row.wrapping_add(0x14))
}
