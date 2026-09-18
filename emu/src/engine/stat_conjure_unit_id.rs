use crate::Fault;

use super::{read_flag, AppContext};

pub fn stat_conjure_unit_id(ctx: &AppContext, team: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    if unit_id < -2 {
        return Ok(-1);
    }

    if read_flag(ctx, (team as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 == 0 {
        return Ok(-1);
    }

    ctx.i32_at(((unit_id.wrapping_add(2) as u32 as i64) * 0x760 + (form as i64) * 0x1d8 + 0x9e720) as usize)
}
