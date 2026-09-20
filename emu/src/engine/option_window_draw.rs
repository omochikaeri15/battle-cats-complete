use crate::Fault;

use super::AppContext;

pub fn option_window_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.ui().ok_or(Fault::host_missing())?.option_window_draw();

    Ok(())
}
