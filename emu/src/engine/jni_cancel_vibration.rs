use crate::Fault;

use super::AppContext;

pub fn jni_cancel_vibration(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.platform().ok_or(Fault::HostMissing { site: "jni_cancel_vibration" })?.cancel_vibration();

    Ok(())
}
