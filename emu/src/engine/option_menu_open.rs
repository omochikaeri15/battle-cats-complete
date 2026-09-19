use crate::Fault;

use super::{option_window_build, option_window_build_alt, vibration_clear, AppContext};

pub fn option_menu_open(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.set_block_at::<1>(AppContext::OPTION_WINDOW, [1])?;

    match ctx.i32_at(AppContext::OPTION_WINDOW_KIND)? {
        1 => option_window_build_alt(ctx),
        0 => {
            vibration_clear(ctx);

            option_window_build(ctx)
        }
        _ => Ok(()),
    }
}
