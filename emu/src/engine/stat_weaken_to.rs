use crate::Fault;

use super::{get_talent_value, has_talent, max_i32, read_flag, AppContext, CatStats, EnemyStats};

pub fn stat_weaken_to(ctx: &mut AppContext, faction: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::WEAKEN_TO));
    }


    let mut weaken_to = if ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::WEAKEN_CHANCE))? != 0 { ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::WEAKEN_TO))? } else { 0x64 };

    if has_talent(ctx, faction, unit_id, form, 0x1)? {
        weaken_to = weaken_to.wrapping_sub(get_talent_value(ctx, faction, unit_id, form, 0x1, 2)?);
    }

    Ok(max_i32(weaken_to, 0))
}
