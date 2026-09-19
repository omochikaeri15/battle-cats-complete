use crate::Fault;

use super::{stop_vibration, AppContext};

pub fn vibration_reset(ctx: &mut AppContext) -> Result<(), Fault> {
    stop_vibration(ctx)?;

    ctx.battle_event_latch.kind = 0;

    Ok(())
}
