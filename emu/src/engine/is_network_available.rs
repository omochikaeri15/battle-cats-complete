use crate::Fault;

use super::AppContext;

pub fn is_network_available(ctx: &mut AppContext) -> Result<bool, Fault> {
    Ok(ctx
        .platform()
        .ok_or(Fault::host_missing())?
        .is_network_available())
}
