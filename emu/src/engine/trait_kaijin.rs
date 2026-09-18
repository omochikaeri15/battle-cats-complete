use crate::Fault;

use super::{read_flag, AppContext};

pub fn trait_kaijin(ctx: &AppContext, team: i32, unit_id: i32) -> Result<bool, Fault> {
    if read_flag(ctx, (team as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0 {
        return Ok(false);
    }

    Ok(ctx.i32_at(((unit_id.wrapping_add(2) as i64) * 0x1c4 + 0x2332c0) as usize)? != 0)
}
