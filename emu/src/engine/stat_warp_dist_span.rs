use crate::Fault;

use super::{get_talent_value, read_flag, AppContext};

pub fn stat_warp_dist_span(ctx: &mut AppContext, team: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    if read_flag(ctx, (team as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 == 0 {
        return ctx.i32_at(((unit_id.wrapping_add(2) as i64) * 0x1c4 + 0x233218) as usize);
    }

    let base = ctx.i32_at(((unit_id.wrapping_add(2) as i64) * 0x760 + (form as i64) * 0x1d8 + 0x9e690) as usize)?;

    Ok(get_talent_value(ctx, team, unit_id, form, 0x9, 3)?.wrapping_add(base))
}
