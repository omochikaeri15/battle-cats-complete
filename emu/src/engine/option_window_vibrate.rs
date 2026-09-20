use crate::Fault;

use super::{
    AppContext, dialog_set_back_button, dialog_show, has_vibrator, new_button_node,
    option_window_vibrate_error, play_sound, query_localizable, sound_manager, ui_node_set_cut,
    vibration_sample,
};

pub fn option_window_vibrate(ctx: &mut AppContext, button: i32, event: i32) -> Result<(), Fault> {
    match event {
        3 => {
            if has_vibrator(ctx)? != 0 {
                play_sound(sound_manager(ctx)?, 0xb, None);

                let enabled = ctx.u8_at(AppContext::VIBRATION_ENABLED)?;

                ctx.set_block_at::<1>(AppContext::VIBRATION_ENABLED, [enabled ^ 1])?;

                let mut cut = 0x6a;

                if enabled == 0 {
                    vibration_sample(ctx)?;
                    cut = 0x69;
                }

                ui_node_set_cut(new_button_node(&mut ctx.buttons, button)?, cut)?;
            } else {
                play_sound(sound_manager(ctx)?, 0xf, None);

                let text = query_localizable(ctx, b"vibration_error_popup");
                let dialog =
                    dialog_show(ctx, &text, 0, 0, 4, Some(option_window_vibrate_error))?;

                dialog_set_back_button(ctx, dialog, 0)?;
            }
        }
        0 => play_sound(sound_manager(ctx)?, 0xa, None),
        _ => {}
    }

    Ok(())
}
