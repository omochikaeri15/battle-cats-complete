use crate::{Fault, ops};

use super::{
    AppContext, map_uses_item_cost, dialog_set_back_button, dialog_show_kind2,
    ex_redirect_check_a, ex_redirect_check_b, get_map_type, is_aku_final_map, is_ex_map_68,
    is_ex_option_target, new_button_node, option_window_return_confirm, play_sound,
    query_localizable, sound_manager, std_string_from_cstr, ui_node_get_child,
    ui_node_set_visible,
};

pub fn option_window_return_to_menu(
    ctx: &mut AppContext,
    button: i32,
    event: i32,
) -> Result<(), Fault> {
    ui_node_set_visible(
        ui_node_get_child(new_button_node(&mut ctx.buttons, button)?, 1)?,
        0,
    );
    ui_node_set_visible(
        ui_node_get_child(new_button_node(&mut ctx.buttons, button)?, 2)?,
        0,
    );

    match event {
        0 => play_sound(sound_manager(ctx)?, 0xa, None),
        3 => play_sound(sound_manager(ctx)?, 0xb, None),
        2 => {
            let half = ops::div_2(ctx.i32_at(AppContext::BATTLE_TICKS)?);
            let blink = half
                .wrapping_sub(ops::div_2(half).wrapping_mul(2))
                .wrapping_add(1);

            ui_node_set_visible(
                ui_node_get_child(new_button_node(&mut ctx.buttons, button)?, blink)?,
                1,
            );
        }
        4 => {
            ctx.set_block_at::<1>(AppContext::RETURN_CONFIRM_OPEN, [1])?;

            let mut text: Vec<u8> = Vec::new();

            'show: {
                if get_map_type(ctx, 0)? == -11 {
                    text = query_localizable(ctx, b"legendquest_Cancle");
                    break 'show;
                }

                if get_map_type(ctx, 0)? == -21 {
                    text = query_localizable(ctx, b"labyrinth_cancel");
                    break 'show;
                }

                if ctx.u8_at(AppContext::LEADERSHIP_REFUND)? != 0
                    && ctx.i32_at(AppContext::BATTLE_INTRO_FRAME)? >= 0x14a
                {
                    let key = std_string_from_cstr(b"leadershipreturn_pop1");

                    text = query_localizable(ctx, &key);
                    break 'show;
                }

                text.extend_from_slice(&ctx.option_rows[1][0]);

                if ctx.i32_at(AppContext::BATTLE_INTRO_FRAME)? > 0x149 {
                    break 'show;
                }

                if ctx.i32_at(AppContext::CHAPTER_MODE)? == 0x63
                    && !ex_redirect_check_a(ctx)?
                    && !ex_redirect_check_b(ctx)?
                    && !is_ex_map_68(ctx)?
                    && !is_aku_final_map(ctx)?
                    && !is_ex_option_target(ctx)?
                {
                    break 'show;
                }

                if ctx.u8_at(AppContext::SCORE_MODE_FLAG)? != 0 && get_map_type(ctx, 0)? != 4 {
                    break 'show;
                }

                if get_map_type(ctx, 0)? == -6 {
                    break 'show;
                }

                text.extend_from_slice(b"<br>");
                text.extend_from_slice(&ctx.option_rows[1][1]);

                if get_map_type(ctx, 0)? == -10 {
                    let key = std_string_from_cstr(b"nekovitan_battle00");

                    text = query_localizable(ctx, &key);
                    break 'show;
                }

                if map_uses_item_cost(ctx)? {
                    let key = std_string_from_cstr(b"Stage_battle00");

                    text = query_localizable(ctx, &key);
                }
            }

            let dialog =
                dialog_show_kind2(ctx, &text, 0, 0, 4, Some(option_window_return_confirm))?;

            dialog_set_back_button(ctx, dialog, 1)?;
        }
        _ => {}
    }

    Ok(())
}
