use crate::Fault;

use super::{AppContext, game_option_window_draw};

pub fn option_window_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    if ctx.u8_at(AppContext::OPTION_WINDOW)? == 0 {
        return Ok(());
    }

    match ctx.i32_at(AppContext::OPTION_WINDOW_KIND)? {
        1 => game_option_window_draw(ctx),
        0 => {
            ctx.ui().ok_or(Fault::host_missing())?.option_window_draw();

            Ok(())
        }
        _ => Ok(()),
    }
}
