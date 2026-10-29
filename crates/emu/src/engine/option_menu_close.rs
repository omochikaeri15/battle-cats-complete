use crate::Fault;

use super::{
    AppContext, option_window_set_touchable, set_bgm_duck, sound_manager, vibration_reset,
};

pub fn option_menu_close(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.set_block_at::<1>(AppContext::OPTION_WINDOW, [0])?;

    if ctx.i32_at(AppContext::OPTION_WINDOW_KIND)? != 1
        || ctx.u8_at(AppContext::UNIT_INFO_OVERLAY_OPEN)? == 0
    {
        set_bgm_duck(sound_manager(ctx)?, 0x64);
    }

    match ctx.i32_at(AppContext::OPTION_WINDOW_KIND)? {
        1 => option_window_set_touchable(ctx, 0),
        0 => {
            ctx.ui()
                .ok_or(Fault::host_missing())?
                .title_option_window_set_touchable(0);

            vibration_reset(ctx)
        }
        _ => Ok(()),
    }
}
