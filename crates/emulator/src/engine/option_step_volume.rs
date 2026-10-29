use crate::Fault;

use super::{AppContext, get_setting};

pub fn option_step_volume(ctx: &AppContext, step: i32) -> Result<i32, Fault> {
    let mut volumes = [0i32; 4];

    volumes[1] = get_setting(&ctx.settings, b"volume_percentage_s", 0x19)?;
    volumes[2] = get_setting(&ctx.settings, b"volume_percentage_m", 0x32)?;
    volumes[3] = get_setting(&ctx.settings, b"volume_percentage_l", 0x64)?;

    if step as u32 > 3 {
        return Ok(0);
    }

    Ok(volumes[step as u32 as usize])
}
