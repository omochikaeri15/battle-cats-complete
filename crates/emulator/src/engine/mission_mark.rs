use crate::Fault;

use super::AppContext;

pub fn mission_mark(ctx: &mut AppContext, kind: i32, target: i32) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::host_missing())?
        .mission_mark(kind, target);

    Ok(())
}
