use crate::Fault;

use super::{AppContext, CatStats, get_talent_value, max_i32, read_flag};

pub fn stat_cooldown(
    ctx: &mut AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return Ok(0);
    }

    let base = ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::COOLDOWN))?;
    let reduction = get_talent_value(ctx, faction, unit_id, form, 0x1a, 0)?;

    Ok(max_i32(base.wrapping_sub(reduction), 0))
}
