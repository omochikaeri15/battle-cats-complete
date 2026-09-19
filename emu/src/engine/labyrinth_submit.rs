use crate::Fault;

use super::AppContext;

pub fn labyrinth_submit(ctx: &mut AppContext, cleared: i32, units: i32) -> Result<(), Fault> {
    ctx.platform().ok_or(Fault::HostMissing { site: "labyrinth_submit" })?.labyrinth_submit(cleared, units);

    Ok(())
}
