use crate::Fault;

use super::{has_talent, read_flag, AppContext};

pub fn stat_explosion_immune(ctx: &mut AppContext, faction: i32, unit_id: i32, form: i32) -> Result<bool, Fault> {
    if read_flag(ctx, (faction as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 == 0 {
        return Ok(ctx.i32_at(((unit_id.wrapping_add(2) as i64) * 0x1c4 + 0x2332bc) as usize)? != 0);
    }

    if ctx.i32_at(((unit_id.wrapping_add(2) as i64) * 0x760 + (form as i64) * 0x1d8 + 0x9e738) as usize)? != 0 {
        return Ok(true);
    }

    has_talent(ctx, faction, unit_id, form, 0x45)
}
