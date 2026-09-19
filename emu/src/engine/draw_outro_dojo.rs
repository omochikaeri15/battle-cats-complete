use crate::{operation, Fault};

use super::{
    button_bank_find, dialog_draw, dialog_top, draw_context, draw_cut, draw_cut_scaled, draw_number_plain, draw_panel, draw_percent_number, entry_find_by_id,
    fill_rect, get_bottom_inset_logical, get_design_height2, get_drawable_width, get_map_type, glow_set, hit_test_rect, imgcut_get_sprite_cut, min_i32,
    new_button_draw, ranking_rank_by_id, set_tint, touch_is_down, AppContext, DECK_PRESS_SIZE_TABLE, OUTRO_SLIDE_TABLE,
};

const SITE: &str = "draw_outro_dojo";

pub fn draw_outro_dojo(ctx: &mut AppContext, hidden: u8) -> Result<(), Fault> {
    let phase = ctx.i32_at(AppContext::OUTRO_PHASE)?;

    if phase > 0 {
        let frame = ctx.i32_at(AppContext::OUTRO_FRAME)?;
        let step = if (frame as u32) >= 0xc || phase != 1 { 0xc } else { frame };
        let banner = ctx.img004_sheet.clone();
        let banner = banner.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
        let slide = *OUTRO_SLIDE_TABLE
            .get(step as i64 as usize)
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: step as i64, limit: OUTRO_SLIDE_TABLE.len() as i64 })?;
        let x = operation::div_2(get_drawable_width(ctx)?).wrapping_sub(slide).wrapping_add(-0xef);

        draw_cut(draw_context(&mut ctx.draw)?, banner, x, 0xbe, 0);

        let phase = ctx.i32_at(AppContext::OUTRO_PHASE)?;

        if phase >= 2 {
            let frame = ctx.i32_at(AppContext::OUTRO_FRAME)?;
            let step = if (frame as u32) >= 0xc || phase != 2 { 0xc } else { frame };

            glow_set(draw_context(&mut ctx.draw)?, 2);
            set_tint(draw_context(&mut ctx.draw)?, 0x28, 0x28, 0x4d, 0xff);

            let slide = *OUTRO_SLIDE_TABLE
                .get(step as i64 as usize)
                .ok_or(Fault::IndexOutOfRange { site: SITE, index: step as i64, limit: OUTRO_SLIDE_TABLE.len() as i64 })?;
            let left = 0i32.wrapping_sub(slide);
            let width = get_drawable_width(ctx)?;

            fill_rect(draw_context(&mut ctx.draw)?, left, 0x13b, width, 0x37);
            glow_set(draw_context(&mut ctx.draw)?, 0);

            let digits = ctx.img001_sheet.clone();
            let digits = digits.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
            let origin = operation::div_2(get_drawable_width(ctx)?).wrapping_sub(slide) as f32;
            let score = ctx.i32_at(AppContext::SCORE_TOTAL)?;
            let offset = imgcut_get_sprite_cut(banner, 5)?[2].wrapping_add(0x14);
            let lead = imgcut_get_sprite_cut(banner, 4)?[2].wrapping_add(0x14);
            let bounds = draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0, score, offset, origin, 321.0, -1.0, lead, 1, 0)?;
            let cut = imgcut_get_sprite_cut(banner, 5)?[2];
            let x = operation::cvttss2si(bounds.left - cut as f32 + -20.0);

            draw_cut(draw_context(&mut ctx.draw)?, banner, x, 0x144, 5);

            let x = operation::cvttss2si(20.0 + bounds.right);

            draw_cut(draw_context(&mut ctx.draw)?, banner, x, 0x143, 4);

            let phase = ctx.i32_at(AppContext::OUTRO_PHASE)?;

            if phase != 2 && ctx.i32_at(AppContext::NEW_BEST_SCORE)? == 1 {
                let middle = (bounds.right - bounds.left) * 0.5 + bounds.left;
                let cut = imgcut_get_sprite_cut(banner, 6)?[2];
                let x = operation::cvttss2si(middle - operation::div_2(cut) as f32);
                let ticks = ctx.i32_at(AppContext::OUTRO_TICKS)?;
                let beat = ticks.wrapping_sub(operation::div_4(ticks) * 4);
                let blink = (operation::div_2(beat as i8 as i32) as i8).wrapping_add(6) as u8;

                draw_cut(draw_context(&mut ctx.draw)?, banner, x, 0x12c, blink as i32);
            }

            let phase = ctx.i32_at(AppContext::OUTRO_PHASE)?;

            if phase >= 8 {
                let held = min_i32(phase.wrapping_add(-7), 0x1e);
                let alpha = operation::div_30((held << 8).wrapping_sub(held));

                set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, alpha);

                let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
                let width = get_drawable_width(ctx)?;
                let height = get_design_height2(ctx);

                fill_rect(draw_context(&mut ctx.draw)?, 0, top, width, height);
            }
        }
    }

    if get_map_type(ctx, 0)? == 4 && ctx.i32_at(AppContext::OUTRO_PHASE)? >= 0x26 {
        let plate = ctx.outro_event_sheets[1].clone();
        let plate = plate.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
        let width = imgcut_get_sprite_cut(plate, 0)?[2].wrapping_mul(0x86);
        let height = imgcut_get_sprite_cut(plate, 0)?[3].wrapping_mul(0x86);
        let centre = operation::div_2(get_drawable_width(ctx)?);
        let left = centre.wrapping_add(operation::div_neg_200(width));

        draw_cut_scaled(draw_context(&mut ctx.draw)?, plate, left, 0x32, operation::div_100(width), operation::div_100(height), 0);
        set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0xff, 0xff);

        let title = ctx.stage_name_sheet.clone();
        let title = title.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

        draw_cut(draw_context(&mut ctx.draw)?, title, left.wrapping_add(0x2e), 0x48, 0);

        let panel = ctx.outro_event_sheets[0].clone();
        let panel = panel.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
        let inset = left.wrapping_add(0x2b);

        draw_cut(draw_context(&mut ctx.draw)?, panel, inset, 0x95, 0xb);
        draw_panel(draw_context(&mut ctx.draw)?, panel, inset, 0xb5, 0xe5, 0x2c, 1.0, 0x11, 0x12);
        draw_cut(draw_context(&mut ctx.draw)?, panel, left.wrapping_add(0xf4), 0xc9, 0xc);

        let score = ctx.i32_at(AppContext::SCORE_TOTAL)?;

        draw_number_plain(draw_context(&mut ctx.draw)?, panel, 0x18, score, 0, left.wrapping_add(0xf0) as f32, 186.0, -4.0, 0, 2, 0)?;
        draw_cut(draw_context(&mut ctx.draw)?, panel, inset, 0xef, 5);
        draw_panel(draw_context(&mut ctx.draw)?, panel, inset, 0x10f, 0xb7, 0x23, 1.0, 0x14, 0x15);

        let row = left.wrapping_add(0xbd);

        draw_cut(draw_context(&mut ctx.draw)?, panel, row, 0x11a, 0xc);

        let small = ctx.img001_second_sheet.clone();
        let small = small.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
        let map = ctx.i32_at(AppContext::MAP_INDEX)?;
        let entry = entry_find_by_id(&ctx.ranking_entries, map);

        draw_number_plain(draw_context(&mut ctx.draw)?, small, 0xe, entry, 0, row as f32, 275.0, -4.0, 0, 2, 0)?;

        let phase = ctx.i32_at(AppContext::OUTRO_PHASE)?;
        let held = 0x1ei32.wrapping_sub(min_i32(phase.wrapping_add(-0x26), 0x1e));
        let map = ctx.i32_at(AppContext::MAP_INDEX)?;
        let rank = ranking_rank_by_id(&ctx.ranking_entries, map);
        let map = ctx.i32_at(AppContext::MAP_INDEX)?;
        let span = 0x64i32.wrapping_sub(ranking_rank_by_id(&ctx.ranking_entries, map));
        let percent = operation::div_900(held.wrapping_mul(held).wrapping_mul(span)).wrapping_add(rank);
        let gauge = ctx.outro_event_sheets[0].clone();
        let gauge = gauge.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
        let small = ctx.img001_second_sheet.clone();
        let small = small.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

        draw_percent_number(ctx, gauge, small, left.wrapping_add(0x143), 0x50, percent)?;

        if ctx.i32_at(AppContext::OUTRO_PHASE)? >= 0x44 && hidden == 0 {
            let plate = ctx.img101_sheet.clone();
            let plate = plate.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
            let step = ctx.i32_at(AppContext::OUTRO_OK_PRESS)?;
            let bounce = *DECK_PRESS_SIZE_TABLE
                .get(step as i64 as usize)
                .ok_or(Fault::IndexOutOfRange { site: SITE, index: step as i64, limit: DECK_PRESS_SIZE_TABLE.len() as i64 })?;
            let half = operation::div_2(bounce);
            let x = operation::div_2(get_drawable_width(ctx)?).wrapping_sub(half).wrapping_add(-0xbe);
            let y = ctx
                .i32_at(AppContext::LETTERBOX_SHIFT)?
                .wrapping_sub(half)
                .wrapping_sub(get_bottom_inset_logical(ctx)?)
                .wrapping_add(0x22e);

            draw_cut_scaled(draw_context(&mut ctx.draw)?, plate, x, y, bounce.wrapping_add(0x17d), bounce.wrapping_add(0x48), 3);

            let label = ctx.img006_sheet.clone();
            let label = label.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
            let x = operation::div_2(get_drawable_width(ctx)?).wrapping_sub(half).wrapping_add(-0x7f);
            let y = ctx
                .i32_at(AppContext::LETTERBOX_SHIFT)?
                .wrapping_sub(half)
                .wrapping_sub(get_bottom_inset_logical(ctx)?)
                .wrapping_add(0x237);

            draw_cut_scaled(draw_context(&mut ctx.draw)?, label, x, y, bounce.wrapping_add(0xfe), bounce.wrapping_add(0x37), 1);

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

            new_button_draw(ctx, ok, 0, 0)?;

            let share = button_bank_find(&ctx.buttons, 0xc9).ok_or(Fault::NullPointer { site: SITE })?;

            new_button_draw(ctx, share, 0, 0)?;
        }
    }

    if dialog_top(ctx).is_some() {
        let dialog = dialog_top(ctx).ok_or(Fault::NullPointer { site: SITE })?;

        dialog_draw(ctx, dialog, 1)?;
    }

    if get_map_type(ctx, 0)? == 4 {
        return Ok(());
    }

    if ctx.i32_at(AppContext::OUTRO_PHASE)? != 7 || hidden != 0 {
        return Ok(());
    }

    let ok = button_bank_find(&ctx.buttons, 0xc8).ok_or(Fault::NullPointer { site: SITE })?;

    new_button_draw(ctx, ok, 0, 0)?;

    let share = button_bank_find(&ctx.buttons, 0xc9).ok_or(Fault::NullPointer { site: SITE })?;

    new_button_draw(ctx, share, 0, 0)
}
