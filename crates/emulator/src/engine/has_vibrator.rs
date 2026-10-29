use crate::Fault;

use super::AppContext;

pub fn has_vibrator(ctx: &mut AppContext) -> Result<u8, Fault> {
    Ok(ctx
        .platform()
        .ok_or(Fault::host_missing())?
        .has_vibrator() as u8)
}
