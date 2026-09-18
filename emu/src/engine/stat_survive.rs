use crate::Fault;

use super::{get_talent_value, read_flag, AppContext, CatStats, EnemyStats};

pub fn stat_survive(ctx: &mut AppContext, faction: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::SURVIVE));
    }

    let base = ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::SURVIVE))?;

    Ok(get_talent_value(ctx, faction, unit_id, form, 0xb, 0)?.wrapping_add(base))
}
