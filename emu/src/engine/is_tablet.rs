use crate::Fault;

use super::AppContext;

pub fn is_tablet(ctx: &mut AppContext) -> Result<u8, Fault> {
    Ok(ctx
        .platform()
        .ok_or(Fault::host_missing())?
        .is_tablet() as u8)
}
