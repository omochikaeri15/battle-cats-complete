use crate::Fault;

use super::{
    AppContext, new_button_set_enabled, option_window_build, option_window_build_alt, play_sound,
    sound_manager, vibration_clear,
};

pub fn option_window_unit_info_back(
    ctx: &mut AppContext,
    button: i32,
    event: i32,
) -> Result<(), Fault> {
    match event {
        4 => {
            ctx.set_block_at::<1>(AppContext::UNIT_INFO_OVERLAY_OPEN, [0])?;

            let two_lines = ctx.u8_at(AppContext::UNIT_INFO_SAVED_TWO_LINES)?;

            ctx.set_block_at::<1>(AppContext::DECK_TWO_LINES, [two_lines])?;
            ctx.set_block_at::<1>(AppContext::OPTION_MENU_IS_OPEN, [1])?;
            ctx.set_block_at::<1>(AppContext::OPTION_WINDOW, [1])?;

            match ctx.i32_at(AppContext::OPTION_WINDOW_KIND)? {
                1 => option_window_build_alt(ctx)?,
                0 => {
                    vibration_clear(ctx);
                    option_window_build(ctx)?;
                }
                _ => {}
            }

            new_button_set_enabled(&mut ctx.buttons, button, 0)?;
        }
        3 => play_sound(sound_manager(ctx)?, 0xb, None),
        0 => play_sound(sound_manager(ctx)?, 0xa, None),
        _ => {}
    }

    Ok(())
}
