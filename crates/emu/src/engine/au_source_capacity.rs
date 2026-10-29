use crate::Fault;

use super::AppContext;

pub fn au_source_capacity(ctx: &mut AppContext) -> Result<i32, Fault> {
    Ok(ctx.audio().ok_or(Fault::host_missing())?.source_capacity())
}
