use crate::Fault;

use super::{read_flag, AppContext, CatStats};

pub fn stat_metal_killer_percent(ctx: &AppContext, faction: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return Ok(0);
    }

    ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::METAL_KILLER_PERCENT))
}
