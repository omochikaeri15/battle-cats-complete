use crate::Fault;

use super::{read_flag, AppContext, CAT_STATS, CAT_STATS_FORM_STRIDE, CAT_STATS_UNIT_STRIDE, CatStats};

pub fn stat_conjure_unit_id(ctx: &AppContext, faction: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    if unit_id < -2 {
        return Ok(-1);
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return Ok(-1);
    }

    ctx.i32_at(
        ((unit_id.wrapping_add(2) as u32 as i64) * CAT_STATS_UNIT_STRIDE as i64
            + (form as i64) * CAT_STATS_FORM_STRIDE as i64
            + (CAT_STATS + CatStats::CONJURE_UNIT_ID) as i64) as usize,
    )
}
