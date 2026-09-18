use crate::Fault;

use super::{read_flag, AppContext};

pub fn stat_time_before_death(ctx: &AppContext, faction: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    let cell = if read_flag(ctx, (faction as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0 {
        ((unit_id.wrapping_add(2) as i64) * 0x760 + (form as i64) * 0x1d8 + 0x9e64c) as usize
    } else {
        ((unit_id.wrapping_add(2) as i64) * 0x1c4 + 0x2331d4) as usize
    };

    ctx.i32_at(cell)
}
