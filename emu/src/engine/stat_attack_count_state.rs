use crate::Fault;

use super::{read_flag, AppContext, CatStats, EnemyStats};

pub fn stat_attack_count_state(ctx: &AppContext, faction: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    let cell = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        AppContext::cat_stat(unit_id, form, CatStats::ATTACK_COUNT_STATE)
    } else {
        AppContext::enemy_stat(unit_id, EnemyStats::ATTACK_COUNT_STATE)
    };

    ctx.i32_at(cell)
}
