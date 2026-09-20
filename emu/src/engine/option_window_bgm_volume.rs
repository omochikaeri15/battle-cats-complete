use crate::Fault;

use super::{
    AppContext, get_bgm_volume_setting, new_button_node, option_step_volume, option_volume_step,
    play_sound, set_bgm_volume_setting, sound_manager, ui_node_set_cut,
};

pub fn option_window_bgm_volume(
    ctx: &mut AppContext,
    button: i32,
    event: i32,
    cuts: [i32; 4],
) -> Result<(), Fault> {
    let sound = match event {
        0 => 0xa,
        3 => {
            let volume = get_bgm_volume_setting(sound_manager(ctx)?);
            let step = option_volume_step(ctx, volume)?;
            let next = if step != 3 { step.wrapping_add(1) } else { 0 };

            ui_node_set_cut(
                new_button_node(&mut ctx.buttons, button)?,
                cuts[next as u32 as usize],
            )?;

            let volume = option_step_volume(ctx, next)?;

            set_bgm_volume_setting(sound_manager(ctx)?, volume);

            0xb
        }
        _ => return Ok(()),
    };

    play_sound(sound_manager(ctx)?, sound, None);

    Ok(())
}
