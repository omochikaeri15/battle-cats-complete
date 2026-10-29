use crate::Fault;

use super::{AppContext, jni_cancel_vibration};

pub fn stop_vibration(ctx: &mut AppContext) -> Result<(), Fault> {
    jni_cancel_vibration(ctx)
}
