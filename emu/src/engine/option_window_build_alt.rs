use std::rc::Rc;

use crate::{Fault, ops};

use super::{
    AppContext, DECK_SLOT_X_TABLE, OptionPage, get_bgm_volume_setting, get_drawable_width,
    get_se_volume_setting, has_vibrator, imgcut_get_sprite_cut, new_button_register,
    new_button_set_animated, new_button_set_back_key, option_volume_step,
    option_window_bgm_volume, option_window_close, option_window_deck_rows,
    option_window_exit_scene, option_window_return_to_menu, option_window_se_volume,
    option_window_unit_info, option_window_vibrate, sound_manager, ui_node_add_child,
    ui_node_set_anchor, ui_node_set_panel, ui_node_set_sprite, ui_node_set_visible,
    ui_node_set_zoom,
};

const BGM_CUTS: [i32; 4] = [5, 0x61, 0x60, 4];
const SE_CUTS: [i32; 4] = [8, 0x5f, 0x5e, 7];

pub fn option_window_build_alt(ctx: &mut AppContext) -> Result<(), Fault> {
    let page = AppContext::OPTION_WINDOW_PAGE;
    let sheet = Rc::clone(
        ctx.option_window_sheet
            .as_ref()
            .ok_or(Fault::null_pointer())?,
    );

    let left = ops::cvttss2si(get_drawable_width(ctx)?.wrapping_sub(0x1d0) as f32 * 0.5f32);

    ctx.set_i32_at(page + OptionPage::LEFT, left)?;
    ctx.set_i32_at(page + OptionPage::RIGHT, left.wrapping_add(0x1d0))?;
    ctx.set_i32_at(page + OptionPage::TOP, 0x9a)?;
    ctx.set_i32_at(page + OptionPage::BOTTOM, 0x1c1)?;
    ctx.set_i32_at(page + OptionPage::SIDE, 0x27)?;
    ctx.set_i32_at(page + OptionPage::HEADER, 0x73)?;
    ctx.set_i32_at(page + OptionPage::FOOTER, 0x5b)?;

    let tall = if ctx.i32_at(AppContext::TUTORIAL_TWO_ROWS_SEEN)? != 2 {
        0
    } else {
        ctx.set_i32_at(page + OptionPage::TOP, 0x8e)?;
        ctx.set_i32_at(page + OptionPage::BOTTOM, 0x1d9)?;
        ctx.set_i32_at(page + OptionPage::HEADER, 0x5d)?;

        1
    };

    ctx.set_block_at::<1>(page + OptionPage::TALL, [tall])?;

    let width = imgcut_get_sprite_cut(&sheet, 0x15)?[2];
    let height = imgcut_get_sprite_cut(&sheet, 0x15)?[3];
    let right = ctx.i32_at(page + OptionPage::RIGHT)?;
    let top = ctx.i32_at(page + OptionPage::TOP)?;

    let mut node = ui_node_set_sprite(
        &sheet,
        right.wrapping_add(ops::div_2(width)).wrapping_add(-0x3a),
        top.wrapping_add(ops::div_2(height)).wrapping_add(-0x22),
        0x15,
    )?;

    ui_node_set_anchor(&mut node, 1);

    let child = ui_node_add_child(&mut node, ui_node_set_sprite(&sheet, 0, 0, 0x16)?);

    ui_node_set_anchor(child, 1);
    ui_node_set_visible(child, 0);

    let child = ui_node_add_child(&mut node, ui_node_set_sprite(&sheet, 0, 0, 0x17)?);

    ui_node_set_anchor(child, 1);
    ui_node_set_visible(child, 0);

    let close = new_button_register(
        &mut ctx.buttons,
        0x3e8,
        right.wrapping_add(-0x3a),
        top.wrapping_add(-0x22),
        width,
        height,
        Some(node),
        Some(Rc::new(option_window_close)),
    );

    new_button_set_back_key(&mut ctx.buttons, close, 1)?;

    let width = imgcut_get_sprite_cut(&sheet, 0x4a)?[2];
    let height = imgcut_get_sprite_cut(&sheet, 0x4a)?[3];
    let x = ops::cvttsd2si(get_drawable_width(ctx)? as f64 * 0.5 + -200.0);
    let mut row = 0x64i32;

    if ctx.u8_at(page + OptionPage::TALL)? == 0 {
        row = 0x7c;
    }

    let y = row.wrapping_add(ctx.i32_at(page + OptionPage::TOP)?);

    let mut node = ui_node_set_sprite(
        &sheet,
        ops::div_2(width).wrapping_add(x),
        ops::div_2(height).wrapping_add(y),
        0x4a,
    )?;

    ui_node_set_anchor(&mut node, 1);

    let child = ui_node_add_child(&mut node, ui_node_set_sprite(&sheet, 0, 0, 0x49)?);

    ui_node_set_anchor(child, 1);

    let child = ui_node_add_child(&mut node, ui_node_set_sprite(&sheet, 0, 0, 0x1e)?);

    ui_node_set_anchor(child, 1);
    ui_node_set_zoom(ui_node_set_visible(child, 0), 2.31, 2.0);

    let child = ui_node_add_child(&mut node, ui_node_set_sprite(&sheet, 0, 0, 0x1f)?);

    ui_node_set_anchor(child, 1);
    ui_node_set_zoom(ui_node_set_visible(child, 0), 2.31, 2.0);

    new_button_register(
        &mut ctx.buttons,
        0x3e9,
        x,
        y,
        width,
        height,
        Some(node),
        Some(Rc::new(option_window_exit_scene)),
    );

    let width = imgcut_get_sprite_cut(&sheet, 4)?[2];
    let height = imgcut_get_sprite_cut(&sheet, 4)?[3];
    let x = ops::cvttsd2si(get_drawable_width(ctx)? as f64 * 0.5 + -10.0 + 5.0);
    let mut row = 0x64i32;

    if ctx.u8_at(page + OptionPage::TALL)? == 0 {
        row = 0x7c;
    }

    let y = row.wrapping_add(ctx.i32_at(page + OptionPage::TOP)?);
    let volume = get_bgm_volume_setting(sound_manager(ctx)?);
    let step = option_volume_step(ctx, volume)?;

    let mut node = ui_node_set_sprite(
        &sheet,
        ops::div_2(width).wrapping_add(x),
        ops::div_2(height).wrapping_add(y),
        BGM_CUTS[step as u32 as usize],
    )?;

    ui_node_set_anchor(&mut node, 1);

    new_button_register(
        &mut ctx.buttons,
        0x3ea,
        x,
        y,
        width,
        height,
        Some(node),
        Some(Rc::new(|ctx, button, event| {
            option_window_bgm_volume(ctx, button, event, [5, 0x61, 0x60, 4])
        })),
    );

    let width = imgcut_get_sprite_cut(&sheet, 7)?[2];
    let height = imgcut_get_sprite_cut(&sheet, 7)?[3];
    let x = ops::cvttsd2si(get_drawable_width(ctx)? as f64 * 0.5 + 55.0 + 5.0);
    let mut row = 0x64i32;

    if ctx.u8_at(page + OptionPage::TALL)? == 0 {
        row = 0x7c;
    }

    let y = row.wrapping_add(ctx.i32_at(page + OptionPage::TOP)?);
    let volume = get_se_volume_setting(sound_manager(ctx)?);
    let step = option_volume_step(ctx, volume)?;

    let mut node = ui_node_set_sprite(
        &sheet,
        ops::div_2(width).wrapping_add(x),
        ops::div_2(height).wrapping_add(y),
        SE_CUTS[step as u32 as usize],
    )?;

    ui_node_set_anchor(&mut node, 1);

    new_button_register(
        &mut ctx.buttons,
        0x3eb,
        x,
        y,
        width,
        height,
        Some(node),
        Some(Rc::new(|ctx, button, event| {
            option_window_se_volume(ctx, button, event, [8, 0x5f, 0x5e, 7])
        })),
    );

    let width = imgcut_get_sprite_cut(&sheet, 6)?[2];
    let height = imgcut_get_sprite_cut(&sheet, 6)?[3];
    let x = ops::cvttsd2si(get_drawable_width(ctx)? as f64 * 0.5 + -158.0);
    let bottom = ctx.i32_at(page + OptionPage::BOTTOM)?;

    let mut node = ui_node_set_sprite(
        &sheet,
        ops::div_2(width).wrapping_add(x),
        bottom.wrapping_add(ops::div_2(height)).wrapping_add(-0x4c),
        6,
    )?;

    ui_node_set_anchor(&mut node, 1);

    let child = ui_node_add_child(&mut node, ui_node_set_sprite(&sheet, 0, 0, 0xa)?);

    ui_node_set_anchor(child, 1);

    let child = ui_node_add_child(&mut node, ui_node_set_sprite(&sheet, 0, 0, 0x11)?);

    ui_node_set_anchor(child, 1);
    ui_node_set_zoom(ui_node_set_visible(child, 0), 2.0, 2.0);

    let child = ui_node_add_child(&mut node, ui_node_set_sprite(&sheet, 0, 0, 0x12)?);

    ui_node_set_anchor(child, 1);
    ui_node_set_zoom(ui_node_set_visible(child, 0), 2.0, 2.0);

    new_button_register(
        &mut ctx.buttons,
        0x3ed,
        x,
        bottom.wrapping_add(-0x4c),
        width,
        height,
        Some(node),
        Some(Rc::new(option_window_return_to_menu)),
    );

    if ctx.u8_at(page + OptionPage::TALL)? != 0 {
        let width = imgcut_get_sprite_cut(&sheet, 0x4c)?[2];
        let height = imgcut_get_sprite_cut(&sheet, 0x4c)?[3];
        let x = ops::cvttsd2si(get_drawable_width(ctx)? as f64 * 0.5 + -23.0);
        let top = ctx.i32_at(page + OptionPage::TOP)?;
        let half = ops::div_2(width);
        let cut = 0x4di32.wrapping_sub((ctx.u8_at(AppContext::DECK_TWO_LINES)? < 1) as i32);

        let mut node = ui_node_set_sprite(
            &sheet,
            half.wrapping_add(x),
            top.wrapping_add(ops::div_2(height)).wrapping_add(0xae),
            cut,
        )?;

        ui_node_set_anchor(&mut node, 1);

        let y = top.wrapping_add(0xae);
        let mut shift = 0i32;

        if ctx.u8_at(AppContext::DECK_TWO_LINES)? == 0 {
            shift = half;
        }

        let rows = new_button_register(
            &mut ctx.buttons,
            0x3ec,
            shift.wrapping_add(x),
            y,
            half,
            height,
            Some(node),
            Some(Rc::new(move |ctx, button, event| {
                option_window_deck_rows(ctx, button, event, width, x, y)
            })),
        );

        new_button_set_animated(&mut ctx.buttons, rows, 0)?;
    }

    let slot_x = DECK_SLOT_X_TABLE[0];
    let pad = ops::div_2(get_drawable_width(ctx)?.wrapping_sub(0x3c0));
    let base_y = ctx.deck_bar_base_y;
    let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
    let battle = Rc::clone(ctx.img015_sheet.as_ref().ok_or(Fault::null_pointer())?);

    let mut node = ui_node_set_panel(
        &battle,
        slot_x.wrapping_add(pad).wrapping_add(-0x4c),
        base_y.wrapping_add(shift).wrapping_add(0x35),
        0x80,
        0x3f,
        0x153,
        0x154,
        1.0,
    );

    ui_node_set_anchor(&mut node, 1);

    let child = ui_node_add_child(&mut node, ui_node_set_sprite(&battle, 0, 0, 0x157)?);

    ui_node_set_anchor(child, 1);

    let x = slot_x.wrapping_add(pad).wrapping_add(-0x8c);
    let y = base_y.wrapping_add(shift).wrapping_add(0x16);

    new_button_register(
        &mut ctx.buttons,
        0x3ee,
        x,
        y,
        0x80,
        0x3f,
        Some(node),
        Some(Rc::new(move |ctx, button, event| {
            option_window_unit_info(ctx, button, event, x, 0x80, y, 0x3f)
        })),
    );

    let width = imgcut_get_sprite_cut(&sheet, 0x69)?[2];
    let height = imgcut_get_sprite_cut(&sheet, 0x69)?[3];
    let x = ops::cvttsd2si(get_drawable_width(ctx)? as f64 * 0.5 + 120.0 + 5.0);
    let mut row = 0x64i32;

    if ctx.u8_at(page + OptionPage::TALL)? == 0 {
        row = 0x7c;
    }

    let y = row.wrapping_add(ctx.i32_at(page + OptionPage::TOP)?);
    let mut cut = 0x6b;

    if has_vibrator(ctx)? != 0 {
        cut = 0x69i32.wrapping_add((ctx.u8_at(AppContext::VIBRATION_ENABLED)? < 1) as i32);
    }

    let mut node = ui_node_set_sprite(
        &sheet,
        ops::div_2(width).wrapping_add(x),
        ops::div_2(height).wrapping_add(y),
        cut,
    )?;

    ui_node_set_anchor(&mut node, 1);

    new_button_register(
        &mut ctx.buttons,
        0x3f0,
        x,
        y,
        width,
        height,
        Some(node),
        Some(Rc::new(option_window_vibrate)),
    );

    Ok(())
}
