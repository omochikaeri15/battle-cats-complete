use crate::Fault;

use super::{AppContext, EnemyStats, read_flag};

pub fn trait_dojo(ctx: &AppContext, faction: i32, unit_id: i32) -> Result<bool, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        return Ok(false);
    }

    Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_DOJO))? != 0)
}
