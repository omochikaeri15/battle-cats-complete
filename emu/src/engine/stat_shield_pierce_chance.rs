use crate::Fault;

use super::{AppContext, CatStats, get_talent_value, read_flag};

pub fn stat_shield_pierce_chance(
    ctx: &mut AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return Ok(0);
    }

    let base = ctx.i32_at(AppContext::cat_stat(
        unit_id,
        form,
        CatStats::SHIELD_PIERCE_CHANCE,
    ))?;

    Ok(get_talent_value(ctx, faction, unit_id, form, 0x3a, 0)?.wrapping_add(base))
}
