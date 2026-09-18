use crate::Fault;

use super::{read_flag, AppContext};

pub fn stat_use_gudetama_soul(ctx: &AppContext, team: i32, unit_id: i32, form: i32) -> Result<bool, Fault> {
    let cell = if read_flag(ctx, (team as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0 {
        ((unit_id.wrapping_add(2) as i64) * 0x760 + (form as i64) * 0x1d8 + 0x9e67c) as usize
    } else {
        ((unit_id.wrapping_add(2) as i64) * 0x1c4 + 0x233204) as usize
    };

    Ok(ctx.i32_at(cell)? != 0)
}
