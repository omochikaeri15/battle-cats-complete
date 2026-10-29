use crate::Fault;

use super::{AppContext, CatStats, has_talent, read_flag};

pub fn stat_soulstrike(
    ctx: &mut AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
) -> Result<bool, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return Ok(false);
    }

    if ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::SOULSTRIKE))? != 0 {
        return Ok(true);
    }

    has_talent(ctx, faction, unit_id, form, 0x3b)
}
