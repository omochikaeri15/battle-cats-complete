use crate::Fault;

use super::{get_talent_value, max_i32, read_flag, AppContext, CatStats};

pub fn stat_eoc1_cost(ctx: &mut AppContext, faction: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return Ok(0);
    }

    let base = ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::EOC1_COST))?;
    let discount = get_talent_value(ctx, faction, unit_id, form, 0x19, 0)?;

    Ok(max_i32(discount.wrapping_mul(-100).wrapping_add(base), 0))
}
