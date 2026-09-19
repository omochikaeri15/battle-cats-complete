use crate::{operation, Fault};

use super::{
    button_bank_find, dialog_draw, dialog_top, digit_count, draw_context, draw_cut, draw_cut_scaled, draw_labyrinth_gauge, draw_number_plain, draw_panel,
    draw_surface, draw_surface_aligned, fill_rect, get_bottom_inset_logical, get_design_height2, get_drawable_width, get_labyrinth_floor_best,
    get_labyrinth_floor_reached, get_labyrinth_map_id, get_map_type, get_stage_count, get_stage_score, get_text_width, glow_set, has_point_decay, hit_test_rect,
    imgcut_get_sprite_cut, is_score_stage, labyrinth_active, labyrinth_result_ready, map_index_of_map_id, max_i32, min_i32, new_button_draw, set_tint,
    touch_is_down, xor_row46_get, AppContext, Surface, DECK_PRESS_SIZE_TABLE, OUTRO_SLIDE_TABLE, POPUP_GROW_TABLE,
};

const SITE: &str = "draw_outro_xp";

fn slide(step: i32) -> Result<i32, Fault> {
    OUTRO_SLIDE_TABLE
        .get(step as i64 as usize)
        .copied()
        .ok_or(Fault::IndexOutOfRange { site: SITE, index: step as i64, limit: OUTRO_SLIDE_TABLE.len() as i64 })
}

fn stage_flag(ctx: &AppContext) -> Result<i32, Fault> {
    let row = (AppContext::MAP_STAGE_ROWS as i64 + (ctx.i32_at(AppContext::STAGE_ROW)? as i64) * AppContext::MAP_STAGE_ROW_STRIDE as i64) as usize;

    Ok(xor_row46_get(ctx.bytes_from(row)?, 8).ok_or(Fault::IndexOutOfRange { site: SITE, index: 8, limit: 0x2e })? as i32)
}

fn star_blink(ctx: &AppContext) -> Result<i32, Fault> {
    let ticks = ctx.i32_at(AppContext::OUTRO_TICKS)?;
    let beat = ticks.wrapping_sub(operation::div_4(ticks) * 4);

    Ok((operation::div_2(beat as i8 as i32) as i8).wrapping_add(6) as u8 as i32)
}

fn draw_star(ctx: &mut AppContext, left: f32, right: f32, y: i32) -> Result<(), Fault> {
    let banner = ctx.img004_sheet.clone();
    let banner = banner.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let middle = (right - left) * 0.5 + left;
    let half = operation::div_2(imgcut_get_sprite_cut(banner, 6)?[2]);
    let x = operation::cvttss2si(middle - half as f32);
    let blink = star_blink(ctx)?;

    draw_cut(draw_context(&mut ctx.draw)?, banner, x, y, blink);

    Ok(())
}

fn draw_xp_row(ctx: &mut AppContext, origin: f32, first: i32, second: i32) -> Result<(), Fault> {
    let digits = ctx.img001_sheet.clone();
    let digits = digits.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let banner = ctx.img004_sheet.clone();
    let banner = banner.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let doubled = ctx.u8_at(AppContext::OUTRO_VIDEO_WATCHED)?;
    let base = if doubled != 0 { 0x7d } else { 0 };
    let value = ctx.i32_at(AppContext::WIN_XP)? << doubled;
    let offset = imgcut_get_sprite_cut(banner, 1)?[2].wrapping_add(0x14);
    let lead = imgcut_get_sprite_cut(banner, 2)?[2].wrapping_add(0x14);
    let bounds = draw_number_plain(draw_context(&mut ctx.draw)?, digits, base, value, offset, origin, 321.0, -1.0, lead, 1, 0)?;
    let head = imgcut_get_sprite_cut(banner, 1)?[2];

    draw_cut(draw_context(&mut ctx.draw)?, banner, operation::cvttss2si(bounds.left - head as f32 + -20.0), first, 1);
    draw_cut(draw_context(&mut ctx.draw)?, banner, operation::cvttss2si(20.0 + bounds.right), second, 2);

    Ok(())
}

fn draw_score_row(ctx: &mut AppContext, origin: f32, value: i32, head_cut: i32) -> Result<(), Fault> {
    let digits = ctx.img001_sheet.clone();
    let digits = digits.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let banner = ctx.img004_sheet.clone();
    let banner = banner.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let offset = imgcut_get_sprite_cut(banner, 5)?[2].wrapping_add(0x14);
    let lead = imgcut_get_sprite_cut(banner, 4)?[2].wrapping_add(0x14);
    let bounds = draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0, value, offset, origin, 321.0, -1.0, lead, 1, 0)?;
    let head = imgcut_get_sprite_cut(banner, head_cut)?[2];

    draw_cut(draw_context(&mut ctx.draw)?, banner, operation::cvttss2si(bounds.left - head as f32 + -20.0), 0x144, head_cut);
    draw_cut(draw_context(&mut ctx.draw)?, banner, operation::cvttss2si(20.0 + bounds.right), 0x143, 4);

    if ctx.i32_at(AppContext::OUTRO_PHASE)? != 2 && ctx.i32_at(AppContext::NEW_BEST_SCORE)? == 1 {
        draw_star(ctx, bounds.left, bounds.right, 0x12c)?;
    }

    Ok(())
}

fn draw_banner(ctx: &mut AppContext, step: i32) -> Result<(), Fault> {
    let banner = ctx.img004_sheet.clone();
    let banner = banner.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let x = operation::div_2(get_drawable_width(ctx)?).wrapping_sub(slide(step)?).wrapping_add(-0xef);
    let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;

    if chapter == 3 || chapter == 0x63 {
        if stage_flag(ctx)? == -2 {
            draw_cut(draw_context(&mut ctx.draw)?, banner, x, 0x7d, 0);

            return Ok(());
        }

        if is_score_stage(ctx.event_items.as_ref()) && has_point_decay(ctx.event_items.as_ref().ok_or(Fault::NullPointer { site: SITE })?) {
            draw_cut(draw_context(&mut ctx.draw)?, banner, x, 0xbe, 0xc);

            return Ok(());
        }

        draw_cut(draw_context(&mut ctx.draw)?, banner, x, 0xbe, 0);

        return Ok(());
    }

    if get_map_type(ctx, 0)? == -2 || get_map_type(ctx, 0)? == -12 {
        draw_cut(draw_context(&mut ctx.draw)?, banner, x, 0xbe, 0);

        return Ok(());
    }

    if stage_flag(ctx)? == -2 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? == 0 {
        draw_cut(draw_context(&mut ctx.draw)?, banner, x, 0x7d, 0);

        return Ok(());
    }

    draw_cut(draw_context(&mut ctx.draw)?, banner, x, 0xbe, 0);

    Ok(())
}

fn draw_score_fade(ctx: &mut AppContext) -> Result<(), Fault> {
    let frame = ctx.i32_at(AppContext::OUTRO_FRAME)?;
    let shifted = frame.wrapping_add(0x64);
    let step = if (shifted as u32) < 0xd { shifted } else { 0xc };
    let bar = 0i32.wrapping_sub(slide(step)?);
    let mut origin = bar;

    if (shifted as u32) >= 0x21 {
        origin = slide(max_i32(-0x37i32.wrapping_sub(frame), 0))?;
    }

    glow_set(draw_context(&mut ctx.draw)?, 2);
    set_tint(draw_context(&mut ctx.draw)?, 0x28, 0x28, 0x4d, 0xff);

    let width = get_drawable_width(ctx)?;

    fill_rect(draw_context(&mut ctx.draw)?, bar, 0x13b, width, 0x37);
    glow_set(draw_context(&mut ctx.draw)?, 0);

    let across = operation::div_2(get_drawable_width(ctx)?).wrapping_add(origin) as f32;

    draw_xp_row(ctx, across, 0x142, 0x144)
}

fn draw_unlock_row(ctx: &mut AppContext) -> Result<(), Fault> {
    let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
    let outbreak = ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)?;
    let show = match chapter {
        3 | 0x63 => true,
        4..=6 => outbreak == 0,
        _ => false,
    };

    if !show || stage_flag(ctx)? != -2 {
        return Ok(());
    }

    let frame = ctx.i32_at(AppContext::OUTRO_FRAME)?;
    let step = if (frame as u32) >= 0xc || ctx.i32_at(AppContext::OUTRO_PHASE)? != 2 { 0xc } else { frame };

    glow_set(draw_context(&mut ctx.draw)?, 2);
    set_tint(draw_context(&mut ctx.draw)?, 0x28, 0x28, 0x4d, 0xff);

    let bar = 0i32.wrapping_sub(slide(step)?);
    let width = get_drawable_width(ctx)?;

    fill_rect(draw_context(&mut ctx.draw)?, bar, 0xff, width, 0x37);
    glow_set(draw_context(&mut ctx.draw)?, 0);

    let digits = ctx.img001_sheet.clone();
    let digits = digits.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let banner = ctx.img004_sheet.clone();
    let banner = banner.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let origin = operation::div_2(get_drawable_width(ctx)?).wrapping_sub(slide(step)?) as f32;
    let value = ctx.i32_at(AppContext::STAGE_SCORE)?;
    let offset = imgcut_get_sprite_cut(banner, 5)?[2].wrapping_add(0x14);
    let lead = imgcut_get_sprite_cut(banner, 4)?[2].wrapping_add(0x14);
    let bounds = draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0, value, offset, origin, 261.0, -1.0, lead, 1, 0)?;
    let head = imgcut_get_sprite_cut(banner, 5)?[2];

    draw_cut(draw_context(&mut ctx.draw)?, banner, operation::cvttss2si(bounds.left - head as f32 + -20.0), 0x108, 5);
    draw_cut(draw_context(&mut ctx.draw)?, banner, operation::cvttss2si(20.0 + bounds.right), 0x107, 4);

    if ctx.i32_at(AppContext::OUTRO_PHASE)? != 2 && ctx.i32_at(AppContext::NEW_BEST_SCORE)? == 1 {
        draw_star(ctx, bounds.left, bounds.right, 0xf0)?;
    }

    Ok(())
}

fn draw_unlock_popup(ctx: &mut AppContext) -> Result<(), Fault> {
    let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;

    if chapter == 3 || chapter == 0x63 {
        return Ok(());
    }

    let popup = ctx.scene_img005_sheet.clone();
    let popup = popup.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let centre = operation::div_2(get_drawable_width(ctx)?);
    let step = ctx.i32_at(AppContext::REWARD_POP_COUNTER)?;
    let grow = POPUP_GROW_TABLE
        .get(step as i64 as usize)
        .copied()
        .ok_or(Fault::IndexOutOfRange { site: SITE, index: step as i64, limit: POPUP_GROW_TABLE.len() as i64 })?;
    let span = grow.wrapping_mul(0x2b2);
    let rise = grow.wrapping_mul(0xb3);
    let x = operation::div_neg_200(span).wrapping_add(centre);
    let y = operation::div_neg_200(rise).wrapping_add(0x1ea);

    draw_cut_scaled(draw_context(&mut ctx.draw)?, popup, x, y, operation::div_100(span), operation::div_100(rise), 0);

    if (ctx.i32_at(AppContext::REWARD_POP_COUNTER)? as u32) < 4 {
        return Ok(());
    }

    let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
    let unlocked = ctx.i32_at(AppContext::NEXT_STAGE_UNLOCKED)?;
    let name = ctx
        .next_stage_names
        .get(chapter as i64 as usize)
        .and_then(|rows| rows.get(unlocked as i64 as usize))
        .map(|row| row[0].clone())
        .ok_or(Fault::IndexOutOfRange { site: SITE, index: unlocked as i64, limit: 0 })?;
    let span = get_text_width(ctx, &name, 0x1e)?;

    set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

    let text = ctx.label_texts.first().copied().flatten().ok_or(Fault::NullPointer { site: SITE })?;
    let x = operation::div_2(get_drawable_width(ctx)?).wrapping_sub(operation::div_2(span));

    draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&text), x, 0x1bc, 0);

    let frame = ctx.i32_at(AppContext::OUTRO_FRAME)?;
    let beat = frame.wrapping_sub(operation::div_4(frame) * 4);

    if (beat as u32) <= 1 {
        set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0, 0xff);
    } else {
        set_tint(draw_context(&mut ctx.draw)?, 0xff, 0, 0xff, 0xff);
    }

    let caption = ctx.next_stage_caption.clone();
    let span = get_text_width(ctx, &caption, 0x1e)?;
    let text = ctx.label_texts.get(1).copied().flatten().ok_or(Fault::NullPointer { site: SITE })?;
    let x = operation::div_2(get_drawable_width(ctx)?).wrapping_sub(operation::div_2(span));

    draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&text), x, 0x1ec, 0);

    Ok(())
}

fn draw_labyrinth_panel(ctx: &mut AppContext) -> Result<(), Fault> {
    let centre = operation::div_2(get_drawable_width(ctx)?);
    let plate = ctx.outro_event_sheets[1].clone();
    let plate = plate.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

    draw_cut_scaled(draw_context(&mut ctx.draw)?, plate, centre.wrapping_add(-0x151), 0x37, 0x2a2, 0x123, 0);
    set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0xff, 0xff);

    let title = ctx.outro_event_sheets[2].clone();
    let title = title.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

    draw_surface(draw_context(&mut ctx.draw)?, Surface::Sheet(title), centre.wrapping_add(-0x121), 0x5b);

    let panel = ctx.outro_event_sheets[0].clone();
    let panel = panel.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let inset = centre.wrapping_add(-0x94);

    draw_panel(draw_context(&mut ctx.draw)?, panel, inset, 0xb5, 0x9b, 0x25, 1.0, 0x11, 0x12);
    draw_panel(draw_context(&mut ctx.draw)?, panel, inset, 0xf0, 0x9b, 0x25, 1.0, 0x11, 0x12);

    let caption = centre.wrapping_add(-0x115);

    draw_cut(draw_context(&mut ctx.draw)?, panel, caption, 0xb7, 4);

    let floor = get_labyrinth_floor_reached(ctx)?;
    let places = digit_count(floor);
    let map = map_index_of_map_id(get_labyrinth_map_id(ctx)?);
    let total = get_stage_count(ctx, -0x15, map)?;
    let marker = if floor == total { 0x22 } else { 0xa };
    let span = places.wrapping_mul(14);
    let reach = imgcut_get_sprite_cut(panel, marker)?[2].wrapping_add(span);
    let tail = imgcut_get_sprite_cut(panel, 0xb)?[2];
    let anchor = operation::div_2(reach.wrapping_add(tail).wrapping_add(8))
        .wrapping_add(centre)
        .wrapping_add(-0x46)
        .wrapping_sub(imgcut_get_sprite_cut(panel, 0xb)?[2]);

    draw_cut(draw_context(&mut ctx.draw)?, panel, anchor, 0xbf, 0xb);

    let small = ctx.img001_second_sheet.clone();
    let small = small.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

    draw_number_plain(draw_context(&mut ctx.draw)?, small, 0xe, floor, 0, anchor.wrapping_add(-4) as f32, 185.0, -4.0, 0, 2, 0)?;

    let back = 0i32.wrapping_sub(span.wrapping_add(imgcut_get_sprite_cut(panel, marker)?[2]));

    draw_cut(draw_context(&mut ctx.draw)?, panel, back.wrapping_add(anchor).wrapping_add(-8), 0xbf, marker);
    draw_cut(draw_context(&mut ctx.draw)?, panel, caption, 0xf2, 5);

    let best = get_labyrinth_floor_best(ctx)?;
    let places = digit_count(best);
    let span = places.wrapping_mul(14);
    let mark = imgcut_get_sprite_cut(small, 0x7c)?[2];
    let anchor = operation::div_2(span.wrapping_add(mark).wrapping_add(4))
        .wrapping_add(centre)
        .wrapping_add(-0x46)
        .wrapping_sub(imgcut_get_sprite_cut(small, 0x7c)?[2]);

    draw_cut(draw_context(&mut ctx.draw)?, small, anchor, 0xfa, 0x7c);
    draw_number_plain(draw_context(&mut ctx.draw)?, small, 0xe, best, 0, anchor.wrapping_add(-4) as f32, 244.0, -4.0, 0, 2, 0)?;

    let floor = get_labyrinth_floor_reached(ctx)?;
    let gauge = ctx.outro_event_sheets[0].clone();
    let gauge = gauge.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let small = ctx.img001_second_sheet.clone();
    let small = small.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

    draw_labyrinth_gauge(ctx, gauge, small, centre.wrapping_add(0x13), 0x64, floor)
}

fn bounce(ctx: &AppContext) -> Result<i32, Fault> {
    let step = ctx.i32_at(AppContext::OUTRO_OK_PRESS)?;

    DECK_PRESS_SIZE_TABLE
        .get(step as i64 as usize)
        .copied()
        .ok_or(Fault::IndexOutOfRange { site: SITE, index: step as i64, limit: DECK_PRESS_SIZE_TABLE.len() as i64 })
}

fn draw_ok_row(ctx: &mut AppContext) -> Result<(), Fault> {
    let plate = ctx.img101_sheet.clone();
    let plate = plate.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let press = bounce(ctx)?;
    let half = operation::div_2(press);
    let x = operation::div_2(get_drawable_width(ctx)?).wrapping_sub(half).wrapping_add(-0xbe);
    let y = ctx
        .i32_at(AppContext::LETTERBOX_SHIFT)?
        .wrapping_sub(half)
        .wrapping_sub(get_bottom_inset_logical(ctx)?)
        .wrapping_add(0x22e);
    let press = bounce(ctx)?;

    draw_cut_scaled(draw_context(&mut ctx.draw)?, plate, x, y, press.wrapping_add(0x17d), press.wrapping_add(0x48), 3);

    let label = ctx.img006_sheet.clone();
    let label = label.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
    let press = bounce(ctx)?;
    let half = operation::div_2(press);
    let x = operation::div_2(get_drawable_width(ctx)?).wrapping_sub(half).wrapping_add(-0x7f);
    let y = ctx
        .i32_at(AppContext::LETTERBOX_SHIFT)?
        .wrapping_sub(half)
        .wrapping_sub(get_bottom_inset_logical(ctx)?)
        .wrapping_add(0x237);
    let press = bounce(ctx)?;

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

    for id in [0xc8, 0xc9, 0xca] {
        let button = button_bank_find(&ctx.buttons, id).ok_or(Fault::NullPointer { site: SITE })?;

        new_button_draw(ctx, button, 0, 0)?;
    }

    Ok(())
}

pub fn draw_outro_xp(ctx: &mut AppContext, hidden: u8) -> Result<(), Fault> {
    let phase = ctx.i32_at(AppContext::OUTRO_PHASE)?;

    if phase > 0 {
        let frame = ctx.i32_at(AppContext::OUTRO_FRAME)?;
        let step = if (frame as u32) >= 0xc || phase != 1 { 0xc } else { frame };

        draw_banner(ctx, step)?;

        let phase = ctx.i32_at(AppContext::OUTRO_PHASE)?;
        let faded = phase == 2 && ctx.i32_at(AppContext::OUTRO_FRAME)? < 0 && is_score_stage(ctx.event_items.as_ref());

        if faded {
            draw_score_fade(ctx)?;
        } else if phase >= 2 {
            let frame = ctx.i32_at(AppContext::OUTRO_FRAME)?;
            let step = if (frame as u32) >= 0xc || phase != 2 { 0xc } else { frame };

            glow_set(draw_context(&mut ctx.draw)?, 2);
            set_tint(draw_context(&mut ctx.draw)?, 0x28, 0x28, 0x4d, 0xff);

            let bar = if is_score_stage(ctx.event_items.as_ref()) { slide(0xc)? } else { 0i32.wrapping_sub(slide(step)?) };
            let width = get_drawable_width(ctx)?;

            fill_rect(draw_context(&mut ctx.draw)?, bar, 0x13b, width, 0x37);
            glow_set(draw_context(&mut ctx.draw)?, 0);

            let origin = operation::div_2(get_drawable_width(ctx)?).wrapping_sub(slide(step)?) as f32;

            if get_map_type(ctx, 0)? == -6 {
                let value = ctx.i32_at(AppContext::SCORE_TOTAL)?;

                draw_score_row(ctx, origin, value, 5)?;
            } else if is_score_stage(ctx.event_items.as_ref()) {
                let value = get_stage_score(ctx.event_items.as_ref().ok_or(Fault::NullPointer { site: SITE })?);

                draw_score_row(ctx, origin, value, 0xb)?;
            } else {
                draw_xp_row(ctx, origin, 0x142, 0x144)?;
            }

            draw_unlock_row(ctx)?;
        }
    }

    let phase = ctx.i32_at(AppContext::OUTRO_PHASE)?;

    if phase == 5 {
        draw_unlock_popup(ctx)?;
    } else if phase >= 8 {
        let held = min_i32(phase.wrapping_add(-7), 0x1e);
        let alpha = operation::div_30((held << 8).wrapping_sub(held));

        set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, alpha);

        let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let width = get_drawable_width(ctx)?;
        let height = get_design_height2(ctx);

        fill_rect(draw_context(&mut ctx.draw)?, 0, top, width, height);
    }

    if get_map_type(ctx, 0)? == -0x15 && ctx.i32_at(AppContext::OUTRO_PHASE)? >= 0x26 {
        draw_labyrinth_panel(ctx)?;
    }

    if dialog_top(ctx).is_some() {
        let dialog = dialog_top(ctx).ok_or(Fault::NullPointer { site: SITE })?;

        dialog_draw(ctx, dialog, 1)?;
    }

    if ctx.i32_at(AppContext::OUTRO_PHASE)? >= 0x44 && hidden == 0 {
        draw_ok_row(ctx)?;
    }

    if ctx.i32_at(AppContext::OUTRO_PHASE)? != 7 || ctx.u8_at(AppContext::CURTAIN_ACTIVE)? != 0 || hidden != 0 {
        return Ok(());
    }

    if labyrinth_active(ctx)? && !labyrinth_result_ready(ctx)? {
        return Ok(());
    }

    for id in [0xc8, 0xc9] {
        let button = button_bank_find(&ctx.buttons, id).ok_or(Fault::NullPointer { site: SITE })?;

        new_button_draw(ctx, button, 0, 0)?;
    }

    if ctx.u8_at(AppContext::OUTRO_VIDEO_BUTTON)? == 0 {
        return Ok(());
    }

    let video = button_bank_find(&ctx.buttons, 0xcb).ok_or(Fault::NullPointer { site: SITE })?;

    new_button_draw(ctx, video, 0, 0)
}
