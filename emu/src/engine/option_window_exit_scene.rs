use crate::{Fault, ops};

use super::{
    AppContext, new_button_node, play_sound, sound_manager, ui_node_get_child,
    ui_node_set_visible,
};

pub fn option_window_exit_scene(
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
            ctx.set_block_at::<1>(AppContext::SCENE_CHANGE_REQUESTED, [1])?;
            ctx.set_i32_at(AppContext::PENDING_SCENE, 2)?;
        }
        _ => {}
    }

    Ok(())
}
