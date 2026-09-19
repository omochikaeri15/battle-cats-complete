use crate::Fault;

use super::{
    AppContext, analytics_named, button_bank_find, new_button_set_touchable, play_sound,
    show_rewarded_ad, sound_manager,
};

const SITE: &str = "game_lose_update_lambda_1";

pub fn game_lose_update_lambda_1(
    ctx: &mut AppContext,
    _button: i32,
    event: i32,
) -> Result<(), Fault> {
    match event {
        4 => {
            let video =
                button_bank_find(&ctx.buttons, 0xcb).ok_or(Fault::NullPointer { site: SITE })?;

            new_button_set_touchable(&mut ctx.buttons, video, 0)?;
            ctx.set_block_at::<1>(AppContext::OUTRO_VIDEO_BUTTON, [0])?;
            analytics_named(ctx, 0x66, b"BattleContinue", b"")?;
            show_rewarded_ad(ctx, 1)?;
        }
        3 => play_sound(sound_manager(ctx)?, 0xb, None),
        0 => play_sound(sound_manager(ctx)?, 0xa, None),
        _ => {}
    }

    Ok(())
}
