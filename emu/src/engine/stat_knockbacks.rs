use crate::Fault;

use super::{get_talent_value, read_flag, AppContext};

pub fn stat_knockbacks(ctx: &mut AppContext, faction: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    if read_flag(ctx, (faction as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 == 0 {
        return ctx.i32_at(((unit_id.wrapping_add(2) as i64) * 0x1c4 + 0x23310c) as usize);
    }

    let base = ctx.i32_at(((unit_id.wrapping_add(2) as i64) * 0x760 + (form as i64) * 0x1d8 + 0x9e56c) as usize)?;

    Ok(get_talent_value(ctx, faction, unit_id, form, 0x1c, 0)?.wrapping_add(base))
}
