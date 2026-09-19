use crate::Fault;

use super::AppContext;

pub fn option_window_build(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.ui().ok_or(Fault::HostMissing { site: "option_window_build" })?.option_window_build(0);

    Ok(())
}
