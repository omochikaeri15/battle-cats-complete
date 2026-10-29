use crate::Fault;

use super::AppContext;

pub fn web_view_is_open(ctx: &mut AppContext) -> Result<bool, Fault> {
    Ok(ctx
        .platform()
        .ok_or(Fault::host_missing())?
        .web_view_is_open())
}
