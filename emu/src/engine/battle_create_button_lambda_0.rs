use crate::Fault;

use super::{
    AppContext, battle_check_login_bonus, button_bank_find, mission_tracker_refresh,
    new_button_set_pressable, new_button_set_touchable, play_sound, sound_manager,
};

const SITE: &str = "battle_create_button_lambda_0";

pub fn battle_create_button_lambda_0(
    ctx: &mut AppContext,
    button: i32,
    event: i32,
) -> Result<(), Fault> {
    match event {
        4 => {
            mission_tracker_refresh(ctx)?;
            battle_check_login_bonus(ctx)?;

            let share =
                button_bank_find(&ctx.buttons, 0xc9).ok_or(Fault::NullPointer { site: SITE })?;

            new_button_set_touchable(&mut ctx.buttons, share, 0)?;

            let map =
                button_bank_find(&ctx.buttons, 0xc8).ok_or(Fault::NullPointer { site: SITE })?;

            new_button_set_touchable(&mut ctx.buttons, map, 0)?;

            if ctx.u8_at(AppContext::OUTRO_VIDEO_BUTTON)? != 0 {
                let third = button_bank_find(&ctx.buttons, 0xcb)
                    .ok_or(Fault::NullPointer { site: SITE })?;

                new_button_set_touchable(&mut ctx.buttons, third, 0)?;
            }
        }
        3 => {
            play_sound(sound_manager(ctx)?, 0xb, None);
            new_button_set_pressable(&mut ctx.buttons, button, 0)?;
        }
        0 => play_sound(sound_manager(ctx)?, 0xa, None),
        _ => {}
    }

    Ok(())
}
