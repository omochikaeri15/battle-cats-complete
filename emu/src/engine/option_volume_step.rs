use crate::Fault;

use super::{AppContext, get_setting};

pub fn option_volume_step(ctx: &AppContext, volume: i32) -> Result<i32, Fault> {
    if get_setting(&ctx.settings, b"volume_percentage_s", 0x19)? == volume {
        return Ok(1);
    }

    if get_setting(&ctx.settings, b"volume_percentage_m", 0x32)? == volume {
        return Ok(2);
    }

    let large = get_setting(&ctx.settings, b"volume_percentage_l", 0x64)?;

    Ok(((large == volume) as i32).wrapping_mul(3))
}
