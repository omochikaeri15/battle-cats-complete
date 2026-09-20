use crate::Fault;

use super::AppContext;

pub fn mission_tracker_refresh(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::host_missing())?
        .mission_refresh();

    Ok(())
}
