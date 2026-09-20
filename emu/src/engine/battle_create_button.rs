use std::rc::Rc;

use crate::{Fault, ops};

use super::{
    AppContext, battle_create_button_lambda_0, battle_create_button_lambda_1,
    get_bottom_inset_logical, get_drawable_width, get_right_inset_logical, imgcut_get_sprite_cut,
    lose_exit_map_check, new_button_register, new_button_set_enabled, new_button_set_touchable,
    ui_node_add_child, ui_node_set_anchor, ui_node_set_panel, ui_node_set_sprite,
    unlock_popup_is_unlocked,
};

pub fn battle_create_button(ctx: &mut AppContext) -> Result<(), Fault> {
    let sheet = Rc::clone(
        ctx.img001_sheet
            .as_ref()
            .ok_or(Fault::null_pointer())?,
    );
    let width = imgcut_get_sprite_cut(&sheet, 0x78)?[2];
    let height = imgcut_get_sprite_cut(&sheet, 0x78)?[3];
    let drawable = get_drawable_width(ctx)?;
    let right = get_right_inset_logical(ctx)?.wrapping_add(width);
    let left = drawable.wrapping_sub(right);
    let top = 10i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

    let mut node = ui_node_set_sprite(
        &sheet,
        ops::div_2(width).wrapping_add(left).wrapping_add(-10),
        ops::div_2(height).wrapping_add(top),
        0x78,
    )?;

    ui_node_set_anchor(&mut node, 1);

    let map = new_button_register(
        &mut ctx.buttons,
        0xc8,
        drawable.wrapping_sub(right).wrapping_add(-10),
        top,
        width,
        height,
        Some(node),
        Some(Rc::new(battle_create_button_lambda_0)),
    );

    let enabled = ctx.i32_at(AppContext::OUTRO_MAP_LOCKED)? == 0
        && unlock_popup_is_unlocked(ctx, 0x4b)
        && !lose_exit_map_check(ctx)?;
    let map = new_button_set_enabled(&mut ctx.buttons, map, enabled as u8)?;

    new_button_set_touchable(&mut ctx.buttons, map, 0)?;
    ctx.set_block_at::<2>(AppContext::OUTRO_EXIT_DIRECT, [0, 0])?;

    let x = get_drawable_width(ctx)?.wrapping_sub(get_right_inset_logical(ctx)?);
    let y = ctx
        .i32_at(AppContext::LETTERBOX_SHIFT)?
        .wrapping_sub(get_bottom_inset_logical(ctx)?);
    let common = Rc::clone(
        ctx.img039_sheet
            .as_ref()
            .ok_or(Fault::null_pointer())?,
    );

    let mut panel = ui_node_set_panel(
        &common,
        x.wrapping_add(-0x6a),
        y.wrapping_add(0x253),
        0xb2,
        0x30,
        0xf,
        0x10,
        1.0,
    );

    ui_node_set_anchor(&mut panel, 1);

    let label = ui_node_set_sprite(&common, 0, 0, 0xe)?;

    ui_node_set_anchor(ui_node_add_child(&mut panel, label), 1);

    let share = new_button_register(
        &mut ctx.buttons,
        0xc9,
        x.wrapping_add(-0xc3),
        y.wrapping_add(0x22e).wrapping_add(3),
        0xb2,
        0x44,
        Some(panel),
        Some(Rc::new(battle_create_button_lambda_1)),
    );

    new_button_set_touchable(&mut ctx.buttons, share, 0)?;

    Ok(())
}
