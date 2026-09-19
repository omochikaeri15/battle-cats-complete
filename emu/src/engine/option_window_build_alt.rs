use crate::Fault;

use super::AppContext;

pub fn option_window_build_alt(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.ui().ok_or(Fault::HostMissing { site: "option_window_build_alt" })?.option_window_build(1);

    Ok(())
}
