use crate::Fault;

use super::{dialog_close, new_button_set_touchable, play_sound, show_rewarded_ad, sound_manager, AppContext};

pub fn battle_init_win_lambda_0_0(ctx: &mut AppContext, dialog: u64, event: i32, button: i32) -> Result<(), Fault> {
    if event != 5 {
        if event != 2 || button as u32 > 1 {
            return Ok(());
        }

        play_sound(sound_manager(ctx)?, 0xb, None);

        return dialog_close(ctx, dialog);
    }

    match button {
        1 if ctx.ad_button_cleared != 0 => {
            ctx.set_i32_at(AppContext::AD_CONFIRM_DECLINED, 1)?;
        }
        0 => {
            new_button_set_touchable(&mut ctx.buttons, ctx.ad_button_id, 0)?;
            ctx.set_block_at::<1>(AppContext::RESULT_VIDEO_BUTTON, [0])?;

            if ctx.ad_button_cleared != 0 {
                sound_manager(ctx)?.pause_all();
            }

            show_rewarded_ad(ctx, 0)?;
        }
        _ => {}
    }

    Ok(())
}
