use crate::Fault;

use super::AppContext;

pub fn ad_prepare(ctx: &mut AppContext, kind: i32) -> Result<(), Fault> {
    ctx.platform()
        .ok_or(Fault::HostMissing { site: "ad_prepare" })?
        .ad_prepare(kind);

    Ok(())
}
