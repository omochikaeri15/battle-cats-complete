use crate::Fault;

use super::{jni_cancel_vibration, AppContext};

pub fn stop_vibration(ctx: &mut AppContext) -> Result<(), Fault> {
    jni_cancel_vibration(ctx)
}
