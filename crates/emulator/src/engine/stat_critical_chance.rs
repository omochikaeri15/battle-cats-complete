use crate::Fault;

use super::{AppContext, CatStats, EnemyStats, get_cat_combo_bonus, get_talent_value, read_flag};

pub fn stat_critical_chance(
    ctx: &mut AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::CRITICAL_CHANCE));
    }

    let base = ctx.i32_at(AppContext::cat_stat(
        unit_id,
        form,
        CatStats::CRITICAL_CHANCE,
    ))?;
    let chance = get_talent_value(ctx, faction, unit_id, form, 0xd, 0)?.wrapping_add(base);

    if chance <= 0 {
        return Ok(chance);
    }

    Ok(chance.wrapping_add(get_cat_combo_bonus(ctx, &ctx.combo_store, 0x18, unit_id)?))
}
