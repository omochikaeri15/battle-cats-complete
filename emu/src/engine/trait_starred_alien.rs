use crate::Fault;

use super::{read_flag, AppContext, EnemyStats};

pub fn trait_starred_alien(ctx: &AppContext, faction: i32, unit_id: i32) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        return Ok(0);
    }

    ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TRAIT_STARRED_ALIEN))
}
