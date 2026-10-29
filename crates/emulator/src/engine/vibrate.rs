use crate::Fault;

use super::{AppContext, jni_vibrate};

pub fn vibrate(ctx: &mut AppContext, gate: f64, duration: f64, strength: f64) -> Result<(), Fault> {
    if 0.0 >= duration {
        return Ok(());
    }

    let capped = if 30.0 < duration { 30.0 } else { duration };
    let clamped = if 0.02 > capped { 0.02 } else { capped };

    jni_vibrate(ctx, gate, clamped, strength)
}
