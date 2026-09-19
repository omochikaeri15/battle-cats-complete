use crate::Fault;

use super::{AppContext, CatStats, EnemyStats, has_talent, read_flag};

pub fn stat_wave_block(
    ctx: &mut AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
) -> Result<bool, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return Ok(ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::WAVE_BLOCK))? != 0);
    }

    if ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::WAVE_BLOCK))? != 0 {
        return Ok(true);
    }

    has_talent(ctx, faction, unit_id, form, 0x17)
}
