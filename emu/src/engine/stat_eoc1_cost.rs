use crate::Fault;

use super::{get_talent_value, max_i32, read_flag, AppContext};

pub fn stat_eoc1_cost(ctx: &mut AppContext, faction: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    if read_flag(ctx, (faction as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 == 0 {
        return Ok(0);
    }

    let base = ctx.i32_at(((unit_id.wrapping_add(2) as i64) * 0x760 + (form as i64) * 0x1d8 + 0x9e580) as usize)?;
    let discount = get_talent_value(ctx, faction, unit_id, form, 0x19, 0)?;

    Ok(max_i32(discount.wrapping_mul(-100).wrapping_add(base), 0))
}
