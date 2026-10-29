use crate::Fault;

use super::AppContext;

pub fn apply_event_schedule(ctx: &mut AppContext, mode: i32) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::host_missing())?
        .apply_event_schedule(mode);

    Ok(())
}
