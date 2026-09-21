use crate::{Fault, ops};

use super::{
    draw_context, draw_continue_button, draw_cut, draw_cut_f, draw_cut_scaled, draw_model, draw_number_plain, draw_number_scaled, draw_surface_aligned,
    fill_rect, get_anim_len, get_design_height2, get_drawable_width, get_miracle_price, hit_test_rect, imgcut_get_sprite_cut, maanim_execute, obf_value_read,
    set_alpha, set_color, set_tint, sin_deg, touch_is_down, xor_row_get, AppContext, Surface, DECK_PRESS_SIZE_TABLE,
};

pub fn draw_cat_god_menu(ctx: &mut AppContext) -> Result<(), Fault> {
    let fade = ctx.i32_at(AppContext::CAT_GOD_FADE)?;

    set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, fade);

    let top = (-0x64i32).wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
    let width = get_drawable_width(ctx)?;
    let height = get_design_height2(ctx).wrapping_add(0xc8);

    fill_rect(draw_context(&mut ctx.draw)?, 0, top, width, height);

    if ctx.i32_at(AppContext::CAT_GOD_STATE)? >= 2 {
        let sheet = ctx.castle_sheet.clone();
        let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
        let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x1d);
        let y = ctx.i32_at(AppContext::CAT_GOD_OFFSET)?.wrapping_add(ctx.i32_at(AppContext::CAT_GOD_BOB)?).wrapping_add(0x10);

        draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, y, 0x224, 0x1fc, 0);

        let sheet = ctx.img040_sheet.clone();
        let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
        let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0xac);
        let y = ctx.i32_at(AppContext::CAT_GOD_OFFSET)?.wrapping_add(0x11b);

        draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, y, 0x300, 0x100, 0);

        let progress = xor_row_get(ctx.bytes_from(AppContext::CHAPTER_PROGRESS)?, 7).ok_or(Fault::index_out_of_range(7, 10))? as i32;

        if ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? == 4 && progress >= 0x30 {
            let sheet = ctx.img041_sheet.clone();
            let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
            let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x2b7);
            let y = ctx.i32_at(AppContext::CAT_GOD_OFFSET)?.wrapping_add(0x10e);

            draw_cut(draw_context(&mut ctx.draw)?, sheet, x, y, 2);
        }

        if ctx.i32_at(AppContext::CAT_GOD_STATE)? <= 3 {
            if ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? != 4 {
                set_color(draw_context(&mut ctx.draw)?, 0x7f, 0x7f, 0x7f, 0xff);
            }

            for (slot, dx, cut) in [(0usize, 0xf6, 0), (1, 0x1aa, 1), (2, 0x25e, 2)] {
                let sheet = ctx.img042_sheet.clone();
                let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
                let step = ctx.i32_at(AppContext::CAT_GOD_PRESSES.wrapping_add(slot * 4))?;
                let size = *DECK_PRESS_SIZE_TABLE
                    .get(step as i64 as usize)
                    .ok_or(Fault::index_out_of_range(step as i64, DECK_PRESS_SIZE_TABLE.len() as i64))?;
                let half = ops::div_2(size);
                let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(dx).wrapping_sub(half);
                let y = ctx.i32_at(AppContext::CAT_GOD_OFFSET)?.wrapping_sub(half).wrapping_add(0x159);

                draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, y, size.wrapping_add(0x60), size.wrapping_add(0x60), cut);
            }

            set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

            if ctx.u8_at(AppContext::CAT_GOD_CONFIRM_OPEN)? == 0 && ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? == 2 {
                let sheet = ctx.img042_sheet.clone();
                let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
                let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x312);
                let ticks = ctx.i32_at(AppContext::CAT_GOD_OPEN_TICKS)?;
                let y = ops::cvttss2si(sin_deg(ticks.wrapping_mul(30) as f32) * 10.0 + 235.0);

                draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, y, 0x60, 0x60, 0);
            }

            if ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? == 2 {
                set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0, 0xff);

                let text = ctx.label_texts.get(6).copied().flatten().ok_or(Fault::null_pointer())?;
                let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x342);
                let y = ctx.i32_at(AppContext::CAT_GOD_OFFSET)?.wrapping_add(0x1f9);

                draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&text), x, y, 1);
                set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);
            }

            let sheet = ctx.img042_sheet.clone();
            let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
            let step = ctx.i32_at(AppContext::CAT_GOD_PRESSES + 0xc)?;
            let size = *DECK_PRESS_SIZE_TABLE
                .get(step as i64 as usize)
                .ok_or(Fault::index_out_of_range(step as i64, DECK_PRESS_SIZE_TABLE.len() as i64))?;
            let half = ops::div_2(size);
            let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x312).wrapping_sub(half);
            let y = ctx.i32_at(AppContext::CAT_GOD_OFFSET)?.wrapping_sub(half).wrapping_add(0x159);

            draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, y, size.wrapping_add(0x60), size.wrapping_add(0x60), 3);
        } else {
            let sheet = ctx.img042_sheet.clone();
            let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
            let selected = ctx.i32_at(AppContext::CAT_GOD_SELECTED)?;
            let step = ctx.i32_at(AppContext::CAT_GOD_PRESSES.wrapping_add(((selected as i64) * 4) as usize))?;
            let size = *DECK_PRESS_SIZE_TABLE
                .get(step as i64 as usize)
                .ok_or(Fault::index_out_of_range(step as i64, DECK_PRESS_SIZE_TABLE.len() as i64))?;
            let half = ops::div_2(size);
            let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0))
                .wrapping_add(selected.wrapping_mul(0xb4))
                .wrapping_add(0xf6)
                .wrapping_sub(half);
            let y = ctx.i32_at(AppContext::CAT_GOD_OFFSET)?.wrapping_sub(half).wrapping_add(0x159);

            draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, y, size.wrapping_add(0x60), size.wrapping_add(0x60), selected);
        }

        if ctx.i32_at(AppContext::CAT_GOD_STATE)? < 4 {
            let mut step = 0x156i32;

            for slot in 0..4usize {
                let unlocked = ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? == 4;

                if slot != 3 && !unlocked {
                    set_color(draw_context(&mut ctx.draw)?, 0x7f, 0x7f, 0x7f, 0xff);
                }

                let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(step);
                let y = ctx.i32_at(AppContext::CAT_GOD_OFFSET)?.wrapping_add(0x1cf);
                let value = if unlocked {
                    get_miracle_price(ctx, slot as i32)?
                } else if slot == 3 {
                    0
                } else {
                    obf_value_read(&ctx.miracle_levels[slot]) as i32
                };
                let digits = ctx.img001_sheet.clone();
                let digits = digits.as_deref().ok_or(Fault::null_pointer())?;
                let bounds = draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0, value, 0, x as f32, y as f32, 0.0, 0x37, 2, 1)?;
                let icon = ctx.img006_sheet.clone();
                let icon = icon.as_deref().ok_or(Fault::null_pointer())?;
                let across = ops::cvttss2si(bounds.left + -55.0);

                draw_cut_scaled(draw_context(&mut ctx.draw)?, icon, across, y, 0x37, 0x2a, 0x15);
                set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

                step = step.wrapping_add(0xb4);
            }
        }

        if ctx.i32_at(AppContext::CAT_GOD_STATE)? >= 3 {
            set_alpha(draw_context(&mut ctx.draw)?, 0xa5);

            let sheet = ctx.img041_sheet.clone();
            let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
            let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0xd4);
            let y = ctx.i32_at(AppContext::CAT_GOD_OFFSET)?.wrapping_add(0x21);

            draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, y, 0x2bc, 0x70, 1);
            set_alpha(draw_context(&mut ctx.draw)?, 0xff);
            set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

            for (slot, dy) in [(0usize, 0x37), (1, 0x5b)] {
                let text = ctx.label_texts.get(slot).copied().flatten().ok_or(Fault::null_pointer())?;
                let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x243);
                let y = ctx.i32_at(AppContext::CAT_GOD_OFFSET)?.wrapping_add(dy);

                draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&text), x, y, 1);
            }

            let text = ctx.label_texts.get(2).copied().flatten().ok_or(Fault::null_pointer())?;
            let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x22b);

            draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&text), x, 0x12b, 1);

            if ctx.i32_at(AppContext::CAT_GOD_STATE)? <= 4 {
                let sheet = ctx.img006_sheet.clone();
                let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
                let step = ctx.i32_at(AppContext::CAT_GOD_PRESSES + 0x10)?;
                let size = *DECK_PRESS_SIZE_TABLE
                    .get(step as i64 as usize)
                    .ok_or(Fault::index_out_of_range(step as i64, DECK_PRESS_SIZE_TABLE.len() as i64))?;
                let half = ops::div_2(size);

                draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, 4, 0x21di32.wrapping_sub(half), size.wrapping_add(0x5f), size.wrapping_add(0x5f), 9);
                draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, 8, 0x22ei32.wrapping_sub(half), size.wrapping_add(0x54), size.wrapping_add(0x3c), 3);

                let rect = [
                    ctx.i32_at(AppContext::CAT_GOD_CLOSE_RECT)?,
                    ctx.i32_at(AppContext::CAT_GOD_CLOSE_RECT + 4)?,
                    ctx.i32_at(AppContext::CAT_GOD_CLOSE_RECT + 8)?,
                    ctx.i32_at(AppContext::CAT_GOD_CLOSE_RECT + 0xc)?,
                ];

                if touch_is_down(ctx)? != 0 && hit_test_rect(ctx, rect[0], rect[1], rect[2], rect[3])? {
                    let sheet = ctx.img006_sheet.clone();
                    let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
                    let ticks = ctx.i32_at(AppContext::CAT_GOD_TICKS)?;
                    let beat = ticks.wrapping_sub(ops::div_4(ticks) * 4);
                    let cut = (ops::div_2(beat as i8 as i32) as i8).wrapping_add(0xc) as u8;

                    draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, 3, 0x21c, 0x60, 0x60, cut as i32);
                }

                let frame = ctx.img024_sheet.clone();
                let frame = frame.as_deref().ok_or(Fault::null_pointer())?;
                let x = get_drawable_width(ctx)?.wrapping_sub(imgcut_get_sprite_cut(frame, 0xb)?[2]);
                let label_sheet = ctx.img006_sheet.clone();
                let label_sheet = label_sheet.as_deref().ok_or(Fault::null_pointer())?;

                draw_continue_button(ctx, label_sheet, x, 2, 0)?;

                let digits = ctx.img001_sheet.clone();
                let digits = digits.as_deref().ok_or(Fault::null_pointer())?;
                let x = get_drawable_width(ctx)?.wrapping_add(-6) as f32;
                let purse = obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_16_COUNT)?) as i32;

                draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0, purse, 0, x, 592.0, -1.0, 0, 2, 0)?;

                if ctx.i32_at(AppContext::CAT_GOD_STATE)? == 4 {
                    let sheet = ctx.img040_sheet.clone();
                    let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
                    let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x71);

                    draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, 0xc5, 0x300, 0x180, 0);

                    let sheet = ctx.img006_sheet.clone();
                    let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
                    let step = ctx.i32_at(AppContext::CAT_GOD_PRESSES + 0x18)?;
                    let size = *DECK_PRESS_SIZE_TABLE
                        .get(step as i64 as usize)
                        .ok_or(Fault::index_out_of_range(step as i64, DECK_PRESS_SIZE_TABLE.len() as i64))?;
                    let half = ops::div_2(size);
                    let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x323).wrapping_sub(half);

                    draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, 0x9di32.wrapping_sub(half), size.wrapping_add(0x5f), size.wrapping_add(0x5f), 9);

                    let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x328).wrapping_sub(half);

                    draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, 0xaei32.wrapping_sub(half), size.wrapping_add(0x54), size.wrapping_add(0x3c), 0xb);

                    let rect = [
                        ctx.i32_at(AppContext::CAT_GOD_BACK_RECT)?,
                        ctx.i32_at(AppContext::CAT_GOD_BACK_RECT + 4)?,
                        ctx.i32_at(AppContext::CAT_GOD_BACK_RECT + 8)?,
                        ctx.i32_at(AppContext::CAT_GOD_BACK_RECT + 0xc)?,
                    ];

                    if touch_is_down(ctx)? != 0 && hit_test_rect(ctx, rect[0], rect[1], rect[2], rect[3])? {
                        let sheet = ctx.img006_sheet.clone();
                        let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
                        let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x322);
                        let ticks = ctx.i32_at(AppContext::CAT_GOD_TICKS)?;
                        let beat = ticks.wrapping_sub(ops::div_4(ticks) * 4);
                        let cut = (ops::div_2(beat as i8 as i32) as i8).wrapping_add(0xc) as u8;

                        draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, 0x9c, 0x60, 0x60, cut as i32);
                    }

                    let sheet = ctx.img042_sheet.clone();
                    let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
                    let selected = ctx.i32_at(AppContext::CAT_GOD_SELECTED)?;
                    let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0xdc);

                    draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, 0x11c, 0x60, 0x60, selected);

                    let step = ctx.i32_at(AppContext::CAT_GOD_PRESSES + 0x14)?;
                    let size = *DECK_PRESS_SIZE_TABLE
                        .get(step as i64 as usize)
                        .ok_or(Fault::index_out_of_range(step as i64, DECK_PRESS_SIZE_TABLE.len() as i64))?;
                    let half = ops::div_2(size);
                    let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x1a6).wrapping_sub(half);

                    let button = ctx.img101_sheet.clone();
                    let button = button.as_deref().ok_or(Fault::null_pointer())?;

                    draw_cut_scaled(draw_context(&mut ctx.draw)?, button, x, 0x141i32.wrapping_sub(half), size.wrapping_add(0x17d), size.wrapping_add(0x48), 3);

                    let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x1e5).wrapping_sub(half);
                    let caption = ctx.img041_sheet.clone();
                    let caption = caption.as_deref().ok_or(Fault::null_pointer())?;

                    draw_cut_scaled(draw_context(&mut ctx.draw)?, caption, x, 0x149i32.wrapping_sub(half), size.wrapping_add(0xfe), size.wrapping_add(0x37), 0);

                    let selected = ctx.i32_at(AppContext::CAT_GOD_SELECTED)?;
                    let price = get_miracle_price(ctx, selected)?;
                    let purse = obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_16_COUNT)?) as i32;

                    if purse < price {
                        set_color(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);
                        set_alpha(draw_context(&mut ctx.draw)?, 0x7f);

                        let sheet = ctx.img101_sheet.clone();
                        let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
                        let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x1a6).wrapping_sub(half);

                        draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, 0x141i32.wrapping_sub(half), size.wrapping_add(0x17d), size.wrapping_add(0x48), 3);
                        set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);
                        set_alpha(draw_context(&mut ctx.draw)?, 0xff);
                    }

                    set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

                    let rect = [
                        ctx.i32_at(AppContext::CAT_GOD_CONFIRM_RECT)?,
                        ctx.i32_at(AppContext::CAT_GOD_CONFIRM_RECT + 4)?,
                        ctx.i32_at(AppContext::CAT_GOD_CONFIRM_RECT + 8)?,
                        ctx.i32_at(AppContext::CAT_GOD_CONFIRM_RECT + 0xc)?,
                    ];

                    if touch_is_down(ctx)? != 0 && hit_test_rect(ctx, rect[0], rect[1], rect[2], rect[3])? {
                        let sheet = ctx.img101_sheet.clone();
                        let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
                        let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x1a6);
                        let ticks = ctx.i32_at(AppContext::BATTLE_TICKS)?;
                        let beat = ticks.wrapping_sub(ops::div_4(ticks) * 4);
                        let cut = (ops::div_2(beat as i8 as i32) as i8).wrapping_add(4) as u8;

                        draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, 0x141, 0x17d, 0x48, cut as i32);
                    }

                    let unlocked = ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? == 4;
                    let selected = ctx.i32_at(AppContext::CAT_GOD_SELECTED)?;
                    let value = if unlocked {
                        get_miracle_price(ctx, selected)?
                    } else {
                        obf_value_read(&ctx.miracle_levels[selected as i64 as usize]) as i32
                    };
                    let digits = ctx.img001_sheet.clone();
                    let digits = digits.as_deref().ok_or(Fault::null_pointer())?;
                    let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x10c) as f32;
                    let bounds = draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0, value, 0, x, 388.0, -1.0, 0x37, 1, 0)?;
                    let icon = ctx.img006_sheet.clone();
                    let icon = icon.as_deref().ok_or(Fault::null_pointer())?;
                    let across = ops::cvttss2si(bounds.left + -55.0);

                    draw_cut_scaled(draw_context(&mut ctx.draw)?, icon, across, 0x185, 0x37, 0x2a, 0x15);

                    for (slot, dx, y) in [(3usize, 0x1f4, 0xe6), (4, 0x1f3, 0x1d4), (5, 0x1f3, 0x1f8)] {
                        let text = ctx.label_texts.get(slot).copied().flatten().ok_or(Fault::null_pointer())?;
                        let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(dx);

                        draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&text), x, y, 1);
                    }

                    if ctx.i32_at(AppContext::CAT_GOD_STATE)? == 5 {
                        let sheet = ctx.img006_sheet.clone();
                        let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
                        let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x234);
                        let ticks = ctx.i32_at(AppContext::CAT_GOD_OPEN_TICKS)?;
                        let y = ops::cvttss2si(sin_deg(ticks.wrapping_mul(30) as f32) * 10.0 + 211.0);

                        draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, y, 0x60, 0x60, 0);
                    }
                }
            }
        }
    }

    let selected = ctx.i32_at(AppContext::CAT_GOD_SELECTED)?;

    if (selected as u32) > 3 {
        return Ok(());
    }

    let frame = ctx.i32_at(AppContext::CAT_GOD_ANIM_FRAME)?;
    let half = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0));

    match selected {
        0 => {
            if frame < get_anim_len(&ctx.castle_anims[2])? {
                let mut model = std::mem::take(&mut ctx.castle_models[2]);
                let track = std::mem::take(&mut ctx.castle_anims[2]);

                maanim_execute(&mut model, Some(&track), frame, 0)?;
                draw_model(draw_context(&mut ctx.draw)?, &model, half, 0);

                ctx.castle_models[2] = model;
                ctx.castle_anims[2] = track;
            }

            for (index, dx) in [(0usize, -0x12c), (1, 0x12c), (2, 0)] {
                let step = ctx.i32_at(AppContext::CAT_GOD_FRAMES.wrapping_add(index * 4))?;

                if step <= 0 || step >= get_anim_len(&ctx.castle_anims[3])? {
                    continue;
                }

                let mut model = std::mem::take(&mut ctx.castle_models[3]);
                let track = std::mem::take(&mut ctx.castle_anims[3]);

                maanim_execute(&mut model, Some(&track), step, 0)?;
                draw_model(draw_context(&mut ctx.draw)?, &model, half.wrapping_add(dx), 0);

                ctx.castle_models[3] = model;
                ctx.castle_anims[3] = track;
            }
        }
        1 => {
            if frame < get_anim_len(&ctx.castle_anims[4])? {
                let mut model = std::mem::take(&mut ctx.castle_models[4]);
                let track = std::mem::take(&mut ctx.castle_anims[4]);

                maanim_execute(&mut model, Some(&track), frame, 0)?;
                draw_model(draw_context(&mut ctx.draw)?, &model, half, 0);

                ctx.castle_models[4] = model;
                ctx.castle_anims[4] = track;
            }

            let step = ctx.i32_at(AppContext::CAT_GOD_FRAMES)?;

            if step > 0 && step < get_anim_len(&ctx.castle_anims[5])? {
                let mut model = std::mem::take(&mut ctx.castle_models[5]);
                let track = std::mem::take(&mut ctx.castle_anims[5]);

                maanim_execute(&mut model, Some(&track), step, 0)?;
                draw_model(draw_context(&mut ctx.draw)?, &model, half, 0);

                ctx.castle_models[5] = model;
                ctx.castle_anims[5] = track;
            }

            let alpha = ctx
                .i32_at(AppContext::CAT_GOD_FRAMES + 4)?
                .wrapping_sub(ctx.i32_at(AppContext::CAT_GOD_FRAMES + 8)?);

            set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, alpha);

            let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
            let width = get_drawable_width(ctx)?;
            let height = get_design_height2(ctx);

            fill_rect(draw_context(&mut ctx.draw)?, 0, top, width, height);
        }
        2 => {
            if frame < get_anim_len(&ctx.castle_anims[0])? {
                let mut model = std::mem::take(&mut ctx.castle_models[0]);
                let track = std::mem::take(&mut ctx.castle_anims[0]);

                maanim_execute(&mut model, Some(&track), frame, 0)?;
                draw_model(draw_context(&mut ctx.draw)?, &model, half, 0);

                ctx.castle_models[0] = model;
                ctx.castle_anims[0] = track;
            }

            for index in 0..10usize {
                let step = ctx.i32_at(AppContext::CAT_GOD_FRAMES.wrapping_add(index * 4))?;

                if step <= 0 || step >= get_anim_len(&ctx.castle_anims[1])? {
                    continue;
                }

                let column = ctx.i32_at(AppContext::CAT_GOD_STATUE_COLUMN.wrapping_add(index * 4))?;
                let row = ctx.i32_at(AppContext::CAT_GOD_STATUE_ROW.wrapping_add(index * 4))?;
                let mut model = std::mem::take(&mut ctx.castle_models[1]);
                let track = std::mem::take(&mut ctx.castle_anims[1]);

                maanim_execute(&mut model, Some(&track), step, 0)?;
                draw_model(draw_context(&mut ctx.draw)?, &model, half.wrapping_add(column.wrapping_mul(40)), row.wrapping_mul(30));

                ctx.castle_models[1] = model;
                ctx.castle_anims[1] = track;
            }
        }
        _ => {
            if frame < get_anim_len(&ctx.castle_anims[6])? {
                let mut model = std::mem::take(&mut ctx.castle_models[6]);
                let track = std::mem::take(&mut ctx.castle_anims[6]);

                maanim_execute(&mut model, Some(&track), frame, 0)?;
                draw_model(draw_context(&mut ctx.draw)?, &model, half, 0);

                ctx.castle_models[6] = model;
                ctx.castle_anims[6] = track;
            }

            let flash = ctx.i32_at(AppContext::CAT_GOD_FLASH_X)?.wrapping_add(8);
            let bob = ctx.i32_at(AppContext::CAT_GOD_BOB)?.wrapping_add(0x40);

            ctx.set_i32_at(AppContext::DRAW_TEMP_1, flash)?;
            ctx.set_i32_at(AppContext::DRAW_TEMP_2, bob)?;
            ctx.set_i32_at(AppContext::DRAW_TEMP_3, 0x1770)?;
            set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

            let alpha = ctx.i32_at(AppContext::CAT_GOD_FRAMES)?;

            set_alpha(draw_context(&mut ctx.draw)?, alpha);

            let digits = ctx.img001_sheet.clone();
            let digits = digits.as_deref().ok_or(Fault::null_pointer())?;
            let across = ctx.i32_at(AppContext::DRAW_TEMP_1)? as f32;
            let baseline = ctx
                .i32_at(AppContext::DRAW_TEMP_2)?
                .wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
            let total = ctx.i32_at(AppContext::DRAW_TEMP_3)?;
            let product = (total as i64).wrapping_mul(0x57619f1);
            let minutes = ((product >> 32) as i32 >> 7).wrapping_add(((product as u64) >> 63) as i32);
            let bounds = draw_number_plain(
                draw_context(&mut ctx.draw)?,
                digits,
                0x53,
                minutes,
                0,
                across,
                baseline.wrapping_add(0x2e) as f32,
                1.0,
                0,
                0x18,
                2,
            )?;
            let mut edge = ops::cvttss2si(bounds.right);
            let digits = ctx.img001_sheet.clone();
            let digits = digits.as_deref().ok_or(Fault::null_pointer())?;

            draw_cut_scaled(draw_context(&mut ctx.draw)?, digits, edge, baseline, 0x1b, 0x2e, 0x5d);

            edge = edge.wrapping_add(0x1b);

            let total = ctx.i32_at(AppContext::DRAW_TEMP_3)?;
            let seconds = ops::div_100(total).wrapping_sub(ops::div_60(ops::div_100(total)).wrapping_mul(0x3c));
            let digits = ctx.img001_sheet.clone();
            let digits = digits.as_deref().ok_or(Fault::null_pointer())?;
            let bounds = draw_number_plain(
                draw_context(&mut ctx.draw)?,
                digits,
                0x53,
                seconds,
                0,
                edge as f32,
                baseline.wrapping_add(0x2e) as f32,
                0.0,
                0,
                0x18,
                2,
            )?;
            let mut edge = ops::cvttss2si(bounds.right);
            let digits = ctx.img001_sheet.clone();
            let digits = digits.as_deref().ok_or(Fault::null_pointer())?;

            draw_cut_f(
                draw_context(&mut ctx.draw)?,
                digits,
                0x5d,
                bounds.right.trunc(),
                baseline.wrapping_add(0x2e) as f32 + -38.333332,
                22.5,
                38.333332,
            );

            edge = edge.wrapping_add(0x16);

            let total = ctx.i32_at(AppContext::DRAW_TEMP_3)?;
            let hundredths = total.wrapping_sub(ops::div_100(total).wrapping_mul(0x64));
            let digits = ctx.img001_sheet.clone();
            let digits = digits.as_deref().ok_or(Fault::null_pointer())?;

            draw_number_scaled(
                draw_context(&mut ctx.draw)?,
                digits,
                0x53,
                hundredths,
                0,
                edge as f32,
                baseline.wrapping_add(0x2e) as f32,
                0.0,
                0.8333333,
                0,
                0x18,
                2,
            )?;
            set_alpha(draw_context(&mut ctx.draw)?, 0xff);
            set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);
        }
    }

    Ok(())
}
