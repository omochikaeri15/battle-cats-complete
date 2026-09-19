use crate::Fault;

use super::AppContext;

pub fn get_powerup(ctx: &AppContext, powerup: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(((powerup as i64) * 4 + AppContext::POWERUPS as i64) as usize)? != 0)
}
