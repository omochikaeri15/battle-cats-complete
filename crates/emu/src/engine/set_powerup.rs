use crate::Fault;

use super::AppContext;

pub fn set_powerup(ctx: &mut AppContext, powerup: i32, value: i32) -> Result<(), Fault> {
    ctx.set_i32_at(
        ((powerup as i64) * 4 + AppContext::POWERUPS as i64) as usize,
        value,
    )?;

    if powerup == 0 && value as u8 == 0 {
        ctx.set_block_at(AppContext::SPEED_UP_LATCH, [0u8; 1])?;
    }

    Ok(())
}
