use crate::Fault;

use super::{has_talent, read_flag, AppContext, CatStats};

pub fn stat_massive_damage(ctx: &mut AppContext, faction: i32, unit_id: i32, form: i32) -> Result<bool, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return Ok(false);
    }

    if ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::MASSIVE_DAMAGE))? != 0 {
        return Ok(true);
    }

    has_talent(ctx, faction, unit_id, form, 0x7)
}
