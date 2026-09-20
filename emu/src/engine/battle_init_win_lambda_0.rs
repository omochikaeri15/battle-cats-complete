use crate::Fault;

use super::{
    AppContext, analytics_named, battle_init_win_lambda_0_0, dialog_close, dialog_set_back_button,
    dialog_show_kind2, dialog_top, new_button_set_touchable, play_sound, query_localizable,
    sound_manager, substitute_tokens,
};

pub fn battle_init_win_lambda_0(
    ctx: &mut AppContext,
    button: i32,
    event: i32,
) -> Result<(), Fault> {
    match event {
        4 => {}
        3 => {
            play_sound(sound_manager(ctx)?, 0xb, None);

            return Ok(());
        }
        0 => {
            play_sound(sound_manager(ctx)?, 0xa, None);

            return Ok(());
        }
        _ => return Ok(()),
    }

    analytics_named(ctx, 0x66, b"BattleXPx2", b"")?;

    let cleared = ctx.ad_button_cleared;
    let flags = if cleared != 0 {
        new_button_set_touchable(&mut ctx.buttons, button, 0)?;

        let dialog = dialog_top(ctx).ok_or(Fault::null_pointer())?;

        dialog_close(ctx, dialog)?;

        0x24
    } else {
        4
    };
    let key: &[u8] = if ctx.ad_button_cleared != 0 {
        b"ScatCPU_ad_confirm"
    } else {
        b"clear_adreward_1"
    };
    let text = query_localizable(ctx, key);
    let xp = ctx.i32_at(AppContext::WIN_XP)?.to_string();
    let text = substitute_tokens(ctx, &text, &[(b"xp", xp.as_bytes())])?;
    let dialog = dialog_show_kind2(ctx, &text, 0, 0, flags, Some(battle_init_win_lambda_0_0))?;

    dialog_set_back_button(ctx, dialog, 1)?;

    Ok(())
}
