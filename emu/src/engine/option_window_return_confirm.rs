use crate::Fault;

use super::{
    AppContext, battle_check_login_bonus, dialog_close, option_window_set_touchable, play_sound,
    sound_manager,
};

pub fn option_window_return_confirm(
    ctx: &mut AppContext,
    dialog: u64,
    event: i32,
    button: i32,
) -> Result<(), Fault> {
    if event != 2 {
        return Ok(());
    }

    if button == -1 {
        return Ok(());
    }

    play_sound(sound_manager(ctx)?, 0xb, None);
    dialog_close(ctx, dialog)?;

    match button {
        1 => ctx.set_block_at::<1>(AppContext::RETURN_CONFIRM_OPEN, [0])?,
        0 => {
            option_window_set_touchable(ctx, 0)?;

            if ctx.i32_at(AppContext::TUTORIAL_CLEARED)? != 0 {
                return battle_check_login_bonus(ctx);
            }

            ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1])?;
            ctx.set_i32_at(AppContext::CURTAIN_STYLE, 1)?;
        }
        _ => {}
    }

    Ok(())
}
