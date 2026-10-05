use crate::Fault;

use super::{
    AppContext, CatStats, get_global_map_id, get_special_rule, get_talent_value, read_flag,
};

pub fn stat_has_recharge_cut(
    ctx: &mut AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
) -> Result<bool, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return Ok(false);
    }

    let base = ctx.i32_at(AppContext::cat_stat(
        unit_id,
        form,
        CatStats::RECHARGE_CUT,
    ))?;

    if get_talent_value(ctx, faction, unit_id, form, 0x48, 0)?.wrapping_add(base) <= 0 {
        return Ok(false);
    }

    let map_id = get_global_map_id(ctx, 0)?;

    Ok(!get_special_rule(ctx, &ctx.special_rules, map_id, 1)?)
}
