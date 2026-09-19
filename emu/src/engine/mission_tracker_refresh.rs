use crate::Fault;

use super::AppContext;

pub fn mission_tracker_refresh(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::HostMissing {
            site: "mission_tracker_refresh",
        })?
        .mission_refresh();

    Ok(())
}
