use crate::Fault;

use super::{
    AppContext, labyrinth_result_ready, labyrinth_set_result_ready, new_button_node, play_sound,
    sound_manager, ui_node_set_cut,
};

pub fn game_win_update_lambda_17(
    ctx: &mut AppContext,
    button: i32,
    event: i32,
) -> Result<(), Fault> {
    match event {
        3 => {
            play_sound(sound_manager(ctx)?, 0xb, None);

            if labyrinth_result_ready(ctx)? {
                labyrinth_set_result_ready(ctx, 0)?;
                ui_node_set_cut(new_button_node(&mut ctx.buttons, button)?, 0x27)
            } else {
                labyrinth_set_result_ready(ctx, 1)?;
                ui_node_set_cut(new_button_node(&mut ctx.buttons, button)?, 0x26)
            }
        }
        0 => {
            play_sound(sound_manager(ctx)?, 0xa, None);

            Ok(())
        }
        _ => Ok(()),
    }
}
