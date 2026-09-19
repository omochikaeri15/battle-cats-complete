use crate::Fault;

use super::AppContext;

pub fn connecting_indicator_hide(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.platform().ok_or(Fault::HostMissing { site: "connecting_indicator_hide" })?.connecting_hide();

    Ok(())
}
