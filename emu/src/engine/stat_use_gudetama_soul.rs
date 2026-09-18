use crate::Fault;

use super::{read_flag, AppContext, CatStats, EnemyStats};

pub fn stat_use_gudetama_soul(ctx: &AppContext, faction: i32, unit_id: i32, form: i32) -> Result<bool, Fault> {
    let cell = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        AppContext::cat_stat(unit_id, form, CatStats::USE_GUDETAMA_SOUL)
    } else {
        AppContext::enemy_stat(unit_id, EnemyStats::USE_GUDETAMA_SOUL)
    };

    Ok(ctx.i32_at(cell)? != 0)
}
