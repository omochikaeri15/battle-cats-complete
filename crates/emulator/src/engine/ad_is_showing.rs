use crate::Fault;

use super::AppContext;

pub fn ad_is_showing(ctx: &mut AppContext) -> Result<bool, Fault> {
    Ok(ctx.platform().ok_or(Fault::host_missing())?.ad_showing())
}
