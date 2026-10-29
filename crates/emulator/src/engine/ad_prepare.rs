use crate::Fault;

use super::AppContext;

pub fn ad_prepare(ctx: &mut AppContext, kind: i32) -> Result<(), Fault> {
    ctx.platform()
        .ok_or(Fault::host_missing())?
        .ad_prepare(kind);

    Ok(())
}
