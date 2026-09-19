use crate::Fault;

use super::AppContext;

pub fn is_tablet(ctx: &mut AppContext) -> Result<u8, Fault> {
    Ok(ctx.platform().ok_or(Fault::HostMissing { site: "is_tablet" })?.is_tablet() as u8)
}
