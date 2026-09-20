use crate::Fault;

use super::AppContext;

pub fn check_medals(ctx: &mut AppContext, kind: i32) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::host_missing())?
        .check_medals(kind);

    Ok(())
}
