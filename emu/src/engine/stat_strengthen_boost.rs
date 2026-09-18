use crate::Fault;

use super::{get_talent_value, read_flag, AppContext, CatStats, EnemyStats};

pub fn stat_strengthen_boost(ctx: &mut AppContext, faction: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::STRENGTHEN_BOOST));
    }

    let base = ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::STRENGTHEN_BOOST))?;

    Ok(get_talent_value(ctx, faction, unit_id, form, 0xa, 1)?.wrapping_add(base))
}
