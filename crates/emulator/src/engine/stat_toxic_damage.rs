use crate::Fault;

use super::{AppContext, EnemyStats, read_flag};

pub fn stat_toxic_damage(ctx: &AppContext, faction: i32, unit_id: i32) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        return Ok(0);
    }

    ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::TOXIC_DAMAGE))
}
