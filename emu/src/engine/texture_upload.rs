use crate::Fault;

use super::{AppContext, Imgcut};

pub fn texture_upload(ctx: &mut AppContext, sheet: &Imgcut) -> Result<(), Fault> {
    ctx.assets()
        .ok_or(Fault::host_missing())?
        .upload(&sheet.png);

    Ok(())
}
