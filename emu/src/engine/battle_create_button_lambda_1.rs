use crate::Fault;

use super::{
    AppContext, button_bank_find, jni_share_image, new_button_get_height, new_button_get_width,
    new_button_get_x, new_button_get_y, play_sound, sound_manager,
};

const SITE: &str = "battle_create_button_lambda_1";

pub fn battle_create_button_lambda_1(
    ctx: &mut AppContext,
    _button: i32,
    event: i32,
) -> Result<(), Fault> {
    match event {
        4 => {
            let x = new_button_get_x(
                &ctx.buttons,
                button_bank_find(&ctx.buttons, 0xc9).ok_or(Fault::NullPointer { site: SITE })?,
            )?;
            let y = new_button_get_y(
                &ctx.buttons,
                button_bank_find(&ctx.buttons, 0xc9).ok_or(Fault::NullPointer { site: SITE })?,
            )?;
            let width = new_button_get_width(
                &ctx.buttons,
                button_bank_find(&ctx.buttons, 0xc9).ok_or(Fault::NullPointer { site: SITE })?,
            )?;
            let height = new_button_get_height(
                &ctx.buttons,
                button_bank_find(&ctx.buttons, 0xc9).ok_or(Fault::NullPointer { site: SITE })?,
            )?;

            jni_share_image(ctx, x, y, width, height)?;
        }
        3 => play_sound(sound_manager(ctx)?, 0xb, None),
        0 => play_sound(sound_manager(ctx)?, 0xa, None),
        _ => {}
    }

    Ok(())
}
