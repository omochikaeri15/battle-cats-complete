use crate::Fault;

use super::{AppContext, CatStats, EnemyStats, get_talent_value, read_flag};

pub fn stat_freeze_duration(
    ctx: &mut AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::FREEZE_DURATION));
    }

    let base = ctx.i32_at(AppContext::cat_stat(
        unit_id,
        form,
        CatStats::FREEZE_DURATION,
    ))?;

    Ok(get_talent_value(ctx, faction, unit_id, form, 0x2, 1)?.wrapping_add(base))
}
