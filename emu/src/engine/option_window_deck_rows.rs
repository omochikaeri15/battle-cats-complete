use crate::{Fault, ops};

use super::{
    AppContext, new_button_node, new_button_set_pos, play_sound, sound_manager, ui_node_set_cut,
};

pub fn option_window_deck_rows(
    ctx: &mut AppContext,
    button: i32,
    event: i32,
    width: i32,
    x: i32,
    y: i32,
) -> Result<(), Fault> {
    match event {
        4 => {
            let two_lines = ctx.u8_at(AppContext::DECK_TWO_LINES)? ^ 1;

            ctx.set_block_at::<1>(AppContext::DECK_TWO_LINES, [two_lines])?;

            let cut = 0x4di32.wrapping_sub((ctx.u8_at(AppContext::DECK_TWO_LINES)? < 1) as i32);

            ui_node_set_cut(new_button_node(&mut ctx.buttons, button)?, cut)?;

            let mut shift = 0i32;

            if ctx.u8_at(AppContext::DECK_TWO_LINES)? == 0 {
                shift = ops::div_2(width);
            }

            new_button_set_pos(&mut ctx.buttons, button, shift.wrapping_add(x), y)?;
        }
        3 => play_sound(sound_manager(ctx)?, 0xb, None),
        0 => play_sound(sound_manager(ctx)?, 0xa, None),
        _ => {}
    }

    Ok(())
}
