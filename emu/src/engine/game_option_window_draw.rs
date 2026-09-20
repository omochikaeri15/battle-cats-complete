use std::rc::Rc;

use crate::{Fault, ops};

use super::{
    AppContext, OptionPage, button_bank_find, draw_context, draw_cut, draw_cut_scaled, fill_rect,
    get_design_height2, get_drawable_width, new_button_draw, set_tint, web_view_is_open,
};

pub fn game_option_window_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    let page = AppContext::OPTION_WINDOW_PAGE;

    if web_view_is_open(ctx)? {
        let button = button_bank_find(&ctx.buttons, 0x3ee).ok_or(Fault::null_pointer())?;

        new_button_draw(ctx, button, 0, 0)?;
    }

    set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xb2);

    let shift = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
    let width = get_drawable_width(ctx)?;
    let height = get_design_height2(ctx);

    fill_rect(draw_context(&mut ctx.draw)?, 0, shift, width, height);
    set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

    let sheet = Rc::clone(
        ctx.option_window_sheet
            .as_ref()
            .ok_or(Fault::null_pointer())?,
    );
    let left = ctx.i32_at(page + OptionPage::LEFT)?;
    let right = ctx.i32_at(page + OptionPage::RIGHT)?;
    let top = ctx.i32_at(page + OptionPage::TOP)?;
    let bottom = ctx.i32_at(page + OptionPage::BOTTOM)?;
    let side = ctx.i32_at(page + OptionPage::SIDE)?;
    let header = ctx.i32_at(page + OptionPage::HEADER)?;
    let footer = ctx.i32_at(page + OptionPage::FOOTER)?;
    let tall = ctx.u8_at(page + OptionPage::TALL)?;

    draw_cut_scaled(
        draw_context(&mut ctx.draw)?,
        &sheet,
        side.wrapping_add(left).wrapping_sub(5),
        header.wrapping_add(top).wrapping_add(-0x14),
        right
            .wrapping_add(side.wrapping_add(left).wrapping_add(side).wrapping_neg())
            .wrapping_add(0x14),
        bottom.wrapping_sub(top).wrapping_add(-0x8c),
        0x35,
    );
    draw_cut_scaled(
        draw_context(&mut ctx.draw)?,
        &sheet,
        side.wrapping_add(left).wrapping_add(-5),
        top,
        right
            .wrapping_add(side.wrapping_add(left).wrapping_add(side).wrapping_neg())
            .wrapping_add(0x14),
        header,
        ((tall as i32) << 5).wrapping_add(0x2f),
    );
    draw_cut_scaled(
        draw_context(&mut ctx.draw)?,
        &sheet,
        side.wrapping_add(left).wrapping_add(-5),
        bottom.wrapping_sub(footer),
        right
            .wrapping_add(side.wrapping_add(left).wrapping_add(side).wrapping_neg())
            .wrapping_add(5),
        footer,
        0x31,
    );
    draw_cut_scaled(
        draw_context(&mut ctx.draw)?,
        &sheet,
        left,
        header.wrapping_add(top).wrapping_add(-0x14),
        side,
        bottom.wrapping_sub(top).wrapping_add(-0x8c),
        0x34,
    );
    draw_cut_scaled(
        draw_context(&mut ctx.draw)?,
        &sheet,
        left,
        top,
        0xa4,
        header,
        if tall == 0 { 0x2e } else { 0x50 },
    );
    draw_cut_scaled(
        draw_context(&mut ctx.draw)?,
        &sheet,
        left,
        bottom.wrapping_sub(footer),
        0xa4,
        footer,
        0x30,
    );
    draw_cut_scaled(
        draw_context(&mut ctx.draw)?,
        &sheet,
        right,
        header.wrapping_add(top).wrapping_add(-0x14),
        0i32.wrapping_sub(side),
        bottom.wrapping_sub(top).wrapping_add(-0x8c),
        0x34,
    );
    draw_cut_scaled(
        draw_context(&mut ctx.draw)?,
        &sheet,
        right.wrapping_add(-0x2a),
        top,
        0x2a,
        header,
        if tall == 0 { 0x32 } else { 0x4e },
    );
    draw_cut_scaled(
        draw_context(&mut ctx.draw)?,
        &sheet,
        right.wrapping_add(-0x2a),
        bottom.wrapping_sub(footer),
        0x2a,
        footer,
        0x33,
    );

    let tall = ctx.u8_at(page + OptionPage::TALL)?;
    let half = get_drawable_width(ctx)? as f64 * 0.5;
    let top = ctx.i32_at(page + OptionPage::TOP)?;

    if tall != 0 {
        draw_cut_scaled(
            draw_context(&mut ctx.draw)?,
            &sheet,
            ops::cvttsd2si(half + -87.0),
            top.wrapping_add(0x10),
            0xb3,
            0x31,
            0x10,
        );
    } else {
        draw_cut(
            draw_context(&mut ctx.draw)?,
            &sheet,
            ops::cvttsd2si(half + -101.0),
            top.wrapping_add(0x1d),
            0x10,
        );
    }

    let button = button_bank_find(&ctx.buttons, 0x3e8).ok_or(Fault::null_pointer())?;

    new_button_draw(ctx, button, 0, 0)?;

    let button = button_bank_find(&ctx.buttons, 0x3e9).ok_or(Fault::null_pointer())?;

    new_button_draw(ctx, button, 0, 0)?;

    let button = button_bank_find(&ctx.buttons, 0x3ea).ok_or(Fault::null_pointer())?;

    new_button_draw(ctx, button, 0, 0)?;

    let button = button_bank_find(&ctx.buttons, 0x3eb).ok_or(Fault::null_pointer())?;

    new_button_draw(ctx, button, 0, 0)?;

    let button = button_bank_find(&ctx.buttons, 0x3ed).ok_or(Fault::null_pointer())?;

    new_button_draw(ctx, button, 0, 0)?;

    let button = button_bank_find(&ctx.buttons, 0x3f0).ok_or(Fault::null_pointer())?;

    new_button_draw(ctx, button, 0, 0)?;

    if !web_view_is_open(ctx)? {
        let button = button_bank_find(&ctx.buttons, 0x3ee).ok_or(Fault::null_pointer())?;

        new_button_draw(ctx, button, 0, 0)?;
    }

    if ctx.u8_at(page + OptionPage::TALL)? != 0 {
        let half = get_drawable_width(ctx)? as f64 * 0.5;
        let y = 0xb6i32.wrapping_add(ctx.i32_at(page + OptionPage::TOP)?);

        draw_cut(
            draw_context(&mut ctx.draw)?,
            &sheet,
            ops::cvttsd2si(half + -147.0),
            y,
            0x4b,
        );

        let button = button_bank_find(&ctx.buttons, 0x3ec).ok_or(Fault::null_pointer())?;

        new_button_draw(ctx, button, 0, 0)?;
    }

    Ok(())
}
