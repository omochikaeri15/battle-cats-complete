use crate::Fault;

use super::AppContext;

pub fn jni_vibrate(
    ctx: &mut AppContext,
    gate: f64,
    duration: f64,
    strength: f64,
) -> Result<(), Fault> {
    ctx.platform()
        .ok_or(Fault::host_missing())?
        .vibrate(gate, duration, strength);

    Ok(())
}
