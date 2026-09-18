use crate::Fault;

use super::{get_talent_value, has_talent, max_i32, read_flag, AppContext, CatStats, EnemyStats};

pub fn stat_strengthen_threshold(ctx: &mut AppContext, faction: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::STRENGTHEN_THRESHOLD));
    }

    let cell = AppContext::cat_stat(unit_id, form, CatStats::STRENGTHEN_THRESHOLD);

    let threshold = if ctx.i32_at(cell)? != 0 {
        let threshold = ctx.i32_at(cell)?;

        if !has_talent(ctx, faction, unit_id, form, 0xa)? {
            return Ok(max_i32(threshold, 0));
        }

        threshold
    } else {
        if !has_talent(ctx, faction, unit_id, form, 0xa)? {
            return Ok(max_i32(0, 0));
        }

        0x64
    };

    Ok(max_i32(threshold.wrapping_sub(get_talent_value(ctx, faction, unit_id, form, 0xa, 0)?), 0))
}
