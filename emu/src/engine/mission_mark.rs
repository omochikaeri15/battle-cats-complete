use crate::Fault;

use super::AppContext;

pub fn mission_mark(ctx: &mut AppContext, kind: i32, target: i32) -> Result<(), Fault> {
    ctx.meta().ok_or(Fault::HostMissing { site: "mission_mark" })?.mission_mark(kind, target);

    Ok(())
}
