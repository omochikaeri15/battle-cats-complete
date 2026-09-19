use crate::{operation, Fault};

use super::{
    button_bank_find, draw_context, draw_continue_button, draw_cut, draw_cut_f, draw_cut_scaled, draw_number_plain, draw_surface_aligned, fill_rect,
    get_bottom_inset_logical, get_design_height2, get_drawable_width, glow_set, hit_test_rect, imgcut_get_sprite_cut, new_button_draw, obf_value_read, set_tint,
    set_tint_alpha, touch_is_down, AppContext, Surface, DECK_PRESS_SIZE_TABLE, LOSE_BANNER_SLIDE_TABLE, POPUP_GROW_TABLE,
};

const SITE: &str = "draw_outro_lose";
const BLANK_LINE: &[u8] = "\u{ff20}".as_bytes();

pub fn draw_outro_lose(ctx: &mut AppContext) -> Result<(), Fault> {
    if ctx.u8_at(AppContext::OUTRO_VIDEO_WATCHED)? != 0 {
        set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);

        let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let width = get_drawable_width(ctx)?;
        let height = get_design_height2(ctx);

        fill_rect(draw_context(&mut ctx.draw)?, 0, top, width, height);

        return Ok(());
    }

    let phase = ctx.i32_at(AppContext::OUTRO_PHASE)?;

    if phase >= 2 {
        let frame = ctx.i32_at(AppContext::OUTRO_FRAME)?;
        let held = if frame < 0x50 { frame } else { 0x50 };
        let fade = operation::div_neg_80((held << 8).wrapping_sub(held));

        glow_set(draw_context(&mut ctx.draw)?, 2);

        let level = if phase == 2 { fade.wrapping_add(0xff) } else { 0 };

        set_tint(draw_context(&mut ctx.draw)?, level, level, level, 0xff);

        let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let width = get_drawable_width(ctx)?;
        let height = get_design_height2(ctx);

        fill_rect(draw_context(&mut ctx.draw)?, 0, top, width, height);
        glow_set(draw_context(&mut ctx.draw)?, 0);
    }

    set_tint_alpha(draw_context(&mut ctx.draw)?, 0xff);

    let phase = ctx.i32_at(AppContext::OUTRO_PHASE)?;

    if phase > 0 {
        let frame = ctx.i32_at(AppContext::OUTRO_FRAME)?;
        let step = if (frame as u32) >= 0x2b || phase != 1 { 0x2b } else { frame };
        let banner = ctx.img004_sheet.clone();
        let banner = banner.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
        let x = operation::div_2(get_drawable_width(ctx)?).wrapping_add(-0x98);
        let lift = *LOSE_BANNER_SLIDE_TABLE
            .get(step as i64 as usize)
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: step as i64, limit: LOSE_BANNER_SLIDE_TABLE.len() as i64 })?;
        let y = ctx.i32_at(AppContext::LOSE_BANNER_Y)?.wrapping_add(lift);

        draw_cut(draw_context(&mut ctx.draw)?, banner, x, y, 3);
    }

    let phase = ctx.i32_at(AppContext::OUTRO_PHASE)?;

    if phase == 4 {
        if ctx.i32_at(AppContext::LOSE_TIP_SHOWN)? != 1 {
            return Ok(());
        }

        let popup = ctx.scene_img005_sheet.clone();
        let popup = popup.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
        let centre = operation::div_2(get_drawable_width(ctx)?);
        let step = ctx.i32_at(AppContext::REWARD_POP_COUNTER)?;
        let grow = *POPUP_GROW_TABLE
            .get(step as i64 as usize)
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: step as i64, limit: POPUP_GROW_TABLE.len() as i64 })?;
        let span = grow.wrapping_mul(0x2b2);
        let rise = grow.wrapping_mul(0xe5);
        let x = operation::div_neg_200(span).wrapping_add(centre);
        let y = operation::div_neg_200(rise).wrapping_add(0x1b8);

        draw_cut_scaled(draw_context(&mut ctx.draw)?, popup, x, y, operation::div_100(span), operation::div_100(rise), 0);

        if (ctx.i32_at(AppContext::REWARD_POP_COUNTER)? as u32) >= 4 {
            set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

            let tip = ctx.i32_at(AppContext::LOSE_TIP)?;
            let lines = ctx
                .lose_rows
                .get(tip as i64 as usize)
                .cloned()
                .ok_or(Fault::IndexOutOfRange { site: SITE, index: tip as i64, limit: ctx.lose_rows.len() as i64 })?;

            if !lines.is_empty() {
                let mut room = 2i32;

                for line in lines.iter() {
                    if line.as_slice() == BLANK_LINE {
                        break;
                    }

                    room = room.wrapping_sub(1);
                }

                let mut down = room.wrapping_mul(18).wrapping_add(0x194);

                for (index, line) in lines.iter().enumerate() {
                    if line.as_slice() == BLANK_LINE {
                        break;
                    }

                    let text = ctx.label_texts.get(index + 2).copied().flatten().ok_or(Fault::NullPointer { site: SITE })?;
                    let centre = operation::div_2(get_drawable_width(ctx)?);

                    draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&text), centre, down, 1);

                    down = down.wrapping_add(0x24);
                }
            }
        }

        let plate = ctx.img101_sheet.clone();
        let plate = plate.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
        let step = ctx.i32_at(AppContext::OUTRO_OK_PRESS)?;
        let press = *DECK_PRESS_SIZE_TABLE
            .get(step as i64 as usize)
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: step as i64, limit: DECK_PRESS_SIZE_TABLE.len() as i64 })?;
        let half = operation::div_2(press);
        let x = operation::div_2(get_drawable_width(ctx)?).wrapping_sub(half).wrapping_add(-0xbe);
        let y = ctx
            .i32_at(AppContext::LETTERBOX_SHIFT)?
            .wrapping_sub(half)
            .wrapping_sub(get_bottom_inset_logical(ctx)?)
            .wrapping_add(0x22e);

        draw_cut_scaled(draw_context(&mut ctx.draw)?, plate, x, y, press.wrapping_add(0x17d), press.wrapping_add(0x48), 3);

        let label = ctx.img006_sheet.clone();
        let label = label.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
        let x = operation::div_2(get_drawable_width(ctx)?).wrapping_sub(half).wrapping_add(-0x7f);
        let y = ctx
            .i32_at(AppContext::LETTERBOX_SHIFT)?
            .wrapping_sub(half)
            .wrapping_sub(get_bottom_inset_logical(ctx)?)
            .wrapping_add(0x237);

        draw_cut_scaled(draw_context(&mut ctx.draw)?, label, x, y, press.wrapping_add(0xfe), press.wrapping_add(0x37), 1);

        let rect = [
            ctx.i32_at(AppContext::OUTRO_OK_RECT)?,
            ctx.i32_at(AppContext::OUTRO_OK_RECT + 4)?,
            ctx.i32_at(AppContext::OUTRO_OK_RECT + 8)?,
            ctx.i32_at(AppContext::OUTRO_OK_RECT + 0xc)?,
        ];

        if touch_is_down(ctx)? != 0 && hit_test_rect(ctx, rect[0], rect[1], rect[2], rect[3])? {
            let plate = ctx.img101_sheet.clone();
            let plate = plate.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
            let x = operation::div_2(get_drawable_width(ctx)?).wrapping_add(-0xbe);
            let y = ctx
                .i32_at(AppContext::LETTERBOX_SHIFT)?
                .wrapping_sub(get_bottom_inset_logical(ctx)?)
                .wrapping_add(0x22e);
            let ticks = ctx.i32_at(AppContext::BATTLE_TICKS)?;
            let beat = ticks.wrapping_sub(operation::div_4(ticks) * 4);
            let cut = (operation::div_2(beat as i8 as i32) as i8).wrapping_add(4) as u8;

            draw_cut_scaled(draw_context(&mut ctx.draw)?, plate, x, y, 0x17d, 0x48, cut as i32);
        }

        let ok = button_bank_find(&ctx.buttons, 0xc8).ok_or(Fault::NullPointer { site: SITE })?;

        return new_button_draw(ctx, ok, 0, 0);
    }

    if phase != 3 || ctx.u8_at(AppContext::CURTAIN_ACTIVE)? != 0 {
        return Ok(());
    }

    let popup = ctx.scene_img005_sheet.clone();
    let popup = popup.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let centre = operation::div_2(get_drawable_width(ctx)?);
    let step = ctx.i32_at(AppContext::REWARD_POP_COUNTER)?;
    let grow = *POPUP_GROW_TABLE
        .get(step as i64 as usize)
        .ok_or(Fault::IndexOutOfRange { site: SITE, index: step as i64, limit: POPUP_GROW_TABLE.len() as i64 })?;
    let span = grow.wrapping_mul(0x2b2);
    let rise = grow.wrapping_mul(0xe5);
    let x = operation::div_neg_200(span).wrapping_add(centre);
    let y = operation::div_neg_200(rise).wrapping_add(0x1ce);

    draw_cut_scaled(draw_context(&mut ctx.draw)?, popup, x, y, operation::div_100(span), operation::div_100(rise), 0);

    if (ctx.i32_at(AppContext::REWARD_POP_COUNTER)? as u32) < 4 {
        return Ok(());
    }

    set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

    let lines = ctx
        .warning2_rows
        .get(2)
        .cloned()
        .ok_or(Fault::IndexOutOfRange { site: SITE, index: 2, limit: ctx.warning2_rows.len() as i64 })?;
    let mut shown = 0i32;

    if lines[0] != BLANK_LINE && lines[1] != BLANK_LINE {
        shown = -1;

        if lines[2] != BLANK_LINE {
            shown = i32::from(lines[3] == BLANK_LINE).wrapping_add(-3);
        }
    }

    let mut down = shown.wrapping_mul(18).wrapping_add(0x194);

    for (line, text) in lines.iter().enumerate().take(4) {
        if text == BLANK_LINE {
            break;
        }

        let text = ctx.label_texts.get(line).copied().flatten().ok_or(Fault::NullPointer { site: SITE })?;
        let centre = operation::div_2(get_drawable_width(ctx)?);

        draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&text), centre, down, 1);

        down = down.wrapping_add(0x24);
    }

    let digits = ctx.img001_sheet.clone();
    let digits = digits.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let x = (get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + 825.0) as f32;

    draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0, 0x1e, 0, x, 306.0, -1.0, 0, 2, 0)?;

    let label = ctx.img006_sheet.clone();
    let label = label.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let x = operation::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + 642.0);

    draw_cut_scaled(draw_context(&mut ctx.draw)?, label, x, 0x132, 0x37, 0x2a, 0x15);

    let icon = ctx.img002_sheet.clone();
    let icon = icon.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let x = (get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + 593.0) as f32;

    draw_cut_f(draw_context(&mut ctx.draw)?, icon, 0x2b, x, 320.0, 47.0, 28.0);

    let plate = ctx.img101_sheet.clone();
    let plate = plate.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

    for (counter, dx, cut) in [(AppContext::OUTRO_OK_PRESS, -0xe5, 0), (AppContext::LOSE_NO_PRESS, 0x3d, 0)] {
        let step = ctx.i32_at(counter)?;
        let press = *DECK_PRESS_SIZE_TABLE
            .get(step as i64 as usize)
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: step as i64, limit: DECK_PRESS_SIZE_TABLE.len() as i64 })?;
        let half = operation::div_2(press);
        let x = operation::div_2(get_drawable_width(ctx)?).wrapping_sub(half).wrapping_add(dx);

        draw_cut_scaled(draw_context(&mut ctx.draw)?, plate, x, 0x1e0i32.wrapping_sub(half), press.wrapping_add(0xa8), press.wrapping_add(0x48), cut);
    }

    let label = ctx.img006_sheet.clone();
    let label = label.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

    for (counter, dx, cut) in [(AppContext::OUTRO_OK_PRESS, -0xdc, 4), (AppContext::LOSE_NO_PRESS, 0x46, 5)] {
        let step = ctx.i32_at(counter)?;
        let press = *DECK_PRESS_SIZE_TABLE
            .get(step as i64 as usize)
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: step as i64, limit: DECK_PRESS_SIZE_TABLE.len() as i64 })?;
        let half = operation::div_2(press);
        let x = operation::div_2(get_drawable_width(ctx)?).wrapping_sub(half).wrapping_add(dx);

        draw_cut_scaled(draw_context(&mut ctx.draw)?, label, x, 0x1e8i32.wrapping_sub(half), press.wrapping_add(0x96), press.wrapping_add(0x37), cut);
    }

    let idle = ctx.i32_at(AppContext::OUTRO_OK_PRESS)? | ctx.i32_at(AppContext::LOSE_NO_PRESS)? == 0;
    let mut shift = None;

    if idle && ctx.u8_at(AppContext::TUTORIAL_POPUP_OPEN)? == 0 {
        if touch_is_down(ctx)? != 0 {
            let rect = [
                ctx.i32_at(AppContext::CANNON_RECT)?,
                ctx.i32_at(AppContext::CANNON_RECT + 4)?,
                ctx.i32_at(AppContext::CANNON_RECT + 8)?,
                ctx.i32_at(AppContext::CANNON_RECT + 0xc)?,
            ];

            if hit_test_rect(ctx, rect[0], rect[1], rect[2], rect[3])? {
                shift = Some(-0xe5);
            }
        }

        if shift.is_none() && ctx.u8_at(AppContext::TUTORIAL_POPUP_OPEN)? == 0 && touch_is_down(ctx)? != 0 {
            let rect = [
                ctx.i32_at(AppContext::WORKER_RECT)?,
                ctx.i32_at(AppContext::WORKER_RECT + 4)?,
                ctx.i32_at(AppContext::WORKER_RECT + 8)?,
                ctx.i32_at(AppContext::WORKER_RECT + 0xc)?,
            ];

            if hit_test_rect(ctx, rect[0], rect[1], rect[2], rect[3])? {
                shift = Some(0x3d);
            }
        }
    }

    if let Some(offset) = shift {
        let plate = ctx.img101_sheet.clone();
        let plate = plate.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
        let x = operation::div_2(get_drawable_width(ctx)?).wrapping_add(offset);
        let frame = ctx.i32_at(AppContext::OUTRO_FRAME)?;
        let beat = frame.wrapping_sub(operation::div_4(frame) * 4);
        let cut = (operation::div_2(beat as i8 as i32) as i8).wrapping_add(1) as u8;

        draw_cut_scaled(draw_context(&mut ctx.draw)?, plate, x, 0x1e0, 0xa8, 0x48, cut as i32);
    }

    let frame = ctx.img024_sheet.clone();
    let frame = frame.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let x = get_drawable_width(ctx)?.wrapping_sub(imgcut_get_sprite_cut(frame, 0xb)?[2]);
    let label = ctx.img006_sheet.clone();
    let label = label.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

    draw_continue_button(ctx, label, x, 2, 0)?;

    let digits = ctx.img001_sheet.clone();
    let digits = digits.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let x = get_drawable_width(ctx)?.wrapping_add(-5) as f32;
    let held = obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_16_COUNT)?) as i32;

    draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0, held, 0, x, 592.0, -1.0, 0, 2, 0)?;

    if ctx.u8_at(AppContext::OUTRO_VIDEO_BUTTON)? == 0 {
        return Ok(());
    }

    let video = button_bank_find(&ctx.buttons, 0xcb).ok_or(Fault::NullPointer { site: SITE })?;

    new_button_draw(ctx, video, 0, 0)
}
