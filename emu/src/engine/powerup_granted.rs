use crate::Fault;

use super::AppContext;

pub fn powerup_granted(ctx: &AppContext, powerup: i32) -> Result<bool, Fault> {
    if powerup as u32 >= 6 {
        return Err(Fault::index_out_of_range(powerup as i64, 6));
    }

    Ok(ctx.u8_at(AppContext::POWERUP_GRANTS + 1 + powerup as usize * 0x18)? != 0)
}
