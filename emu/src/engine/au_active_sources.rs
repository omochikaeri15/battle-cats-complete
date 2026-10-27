use crate::Fault;

use super::AppContext;

pub fn au_active_sources(ctx: &mut AppContext) -> Result<i32, Fault> {
    Ok(ctx.audio().ok_or(Fault::host_missing())?.active_sources())
}
