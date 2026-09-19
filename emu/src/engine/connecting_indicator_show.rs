use crate::Fault;

use super::AppContext;

pub fn connecting_indicator_show(ctx: &mut AppContext, text: &[u8]) -> Result<(), Fault> {
    ctx.platform().ok_or(Fault::HostMissing { site: "connecting_indicator_show" })?.connecting_show(text);

    Ok(())
}
