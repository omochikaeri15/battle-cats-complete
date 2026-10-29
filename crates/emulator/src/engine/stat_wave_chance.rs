use crate::Fault;

use super::{AppContext, CatStats, EnemyStats, get_talent_value, read_flag};

pub fn stat_wave_chance(
    ctx: &mut AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
    mini: i32,
) -> Result<i32, Fault> {
    if mini != -1 {
        let mini_flag = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
            ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::MINI_WAVE_FLAG))?
        } else if get_talent_value(ctx, faction, unit_id, form, 0x3e, 0)? != 0 {
            1
        } else {
            ctx.i32_at(AppContext::cat_stat(
                unit_id,
                form,
                CatStats::MINI_WAVE_FLAG,
            ))?
        };

        if mini_flag != mini {
            return Ok(0);
        }
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::WAVE_CHANCE));
    }

    let base = ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::WAVE_CHANCE))?;
    let first = get_talent_value(ctx, faction, unit_id, form, 0x11, 0)?;

    Ok(get_talent_value(ctx, faction, unit_id, form, 0x3e, 0)?
        .wrapping_add(first)
        .wrapping_add(base))
}
