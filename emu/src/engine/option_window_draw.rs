use crate::Fault;

use super::AppContext;

const SITE: &str = "option_window_draw";

pub fn option_window_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.ui().ok_or(Fault::HostMissing { site: SITE })?.option_window_draw();

    Ok(())
}
