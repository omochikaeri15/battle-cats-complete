use crate::Fault;

use super::AppContext;

pub fn jni_cancel_vibration(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.platform()
        .ok_or(Fault::host_missing())?
        .cancel_vibration();

    Ok(())
}
