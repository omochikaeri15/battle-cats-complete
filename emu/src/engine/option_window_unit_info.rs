use std::rc::Rc;

use crate::{Fault, ops};

use super::{
    AppContext, new_button_register, new_button_set_back_key, option_menu_close,
    option_window_unit_info_back, play_sound, sound_manager, ui_node_add_child,
    ui_node_set_anchor, ui_node_set_panel, ui_node_set_sprite, unit_info_select,
};

pub fn option_window_unit_info(
    ctx: &mut AppContext,
    _button: i32,
    event: i32,
    x: i32,
    width: i32,
    y: i32,
    height: i32,
) -> Result<(), Fault> {
    match event {
        4 => {
            ctx.set_block_at::<1>(AppContext::UNIT_INFO_OVERLAY_OPEN, [1])?;

            let two_lines = ctx.u8_at(AppContext::DECK_TWO_LINES)?;

            ctx.set_block_at::<1>(AppContext::UNIT_INFO_SAVED_TWO_LINES, [two_lines])?;
            ctx.set_block_at::<1>(AppContext::DECK_TWO_LINES, [1])?;
            ctx.set_i32_at(AppContext::UNIT_INFO_SLOT, 0)?;

            let slot = ctx.i32_at(AppContext::UNIT_INFO_SLOT)?;

            unit_info_select(ctx, slot)?;
            option_menu_close(ctx)?;
            ctx.set_block_at::<1>(AppContext::OPTION_MENU_IS_OPEN, [0])?;

            let sheet = Rc::clone(ctx.img015_sheet.as_ref().ok_or(Fault::null_pointer())?);
            let mut panel = ui_node_set_panel(
                &sheet,
                ops::div_2(width).wrapping_add(x),
                ops::div_2(height).wrapping_add(y),
                width,
                height,
                0x155,
                0x156,
                1.0,
            );

            ui_node_set_anchor(&mut panel, 1);

            let label = ui_node_set_sprite(&sheet, 0, 0, 0x158)?;

            ui_node_set_anchor(ui_node_add_child(&mut panel, label), 1);

            let back = new_button_register(
                &mut ctx.buttons,
                0x3ef,
                x,
                y,
                width,
                height,
                Some(panel),
                Some(Rc::new(option_window_unit_info_back)),
            );

            new_button_set_back_key(&mut ctx.buttons, back, 1)?;
        }
        3 => play_sound(sound_manager(ctx)?, 0xb, None),
        0 => play_sound(sound_manager(ctx)?, 0xa, None),
        _ => {}
    }

    Ok(())
}
