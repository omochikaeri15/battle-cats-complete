use crate::Fault;

use super::AppContext;

pub fn ads_available(ctx: &mut AppContext) -> Result<bool, Fault> {
    Ok(ctx
        .platform()
        .ok_or(Fault::host_missing())?
        .ads_available())
}
