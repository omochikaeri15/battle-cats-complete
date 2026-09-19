use crate::{Fault, operation};

use super::{
    AppContext, app_on_draw, back_pressed, button_bank_busy, button_bank_find,
    connecting_indicator_show, dialog_close, dialog_show_alt, dialog_top, get_auto_camera_mode,
    get_drawable_width, get_map_type, handle_battle_swipe_pinch, has_inquiry_code, hit_test_rect,
    labyrinth_active, new_button_set_touchable, play_sound, query_localizable, ranking_name_by_id,
    ranking_rank_by_id, ranking_status_by_id, ranking_submit_score, sound_manager,
    string_format_rank_comment, texture_cache_load, touch_is_down, touch_released,
    trial_win_update_lambda_1,
};

const SITE: &str = "trial_win_update";

pub fn trial_win_update(ctx: &mut AppContext) -> Result<bool, Fault> {
    ctx.set_i32_at(AppContext::SPEED, 1)?;

    let mut phase = ctx.i32_at(AppContext::RESULT_PHASE)?;

    if phase != 0 && phase <= 7 {
        handle_battle_swipe_pinch(ctx)?;
        phase = ctx.i32_at(AppContext::RESULT_PHASE)?;
    } else {
        ctx.set_i32_at(AppContext::SWIPE_VELOCITY, 0)?;
    }

    let frame = ctx.i32_at(AppContext::RESULT_FRAME)?.wrapping_add(1);

    ctx.set_i32_at(AppContext::RESULT_FRAME, frame)?;
    ctx.set_i32_at(
        AppContext::RESULT_TICKS,
        ctx.i32_at(AppContext::RESULT_TICKS)?.wrapping_add(1),
    )?;

    if phase == 0 {
        ctx.set_i32_at(
            AppContext::CAMERA_ZOOM,
            ctx.i32_at(AppContext::CAMERA_ZOOM)?.wrapping_add(0x320),
        )?;
        ctx.set_i32_at(
            AppContext::DECK_BAR_SLIDE,
            ctx.i32_at(AppContext::DECK_BAR_SLIDE)?.wrapping_add(0xa),
        )?;

        if get_auto_camera_mode(ctx)? != 0 {
            return Ok(true);
        }

        if ctx.i32_at(AppContext::RESULT_FRAME)? < 0x14 {
            return Ok(true);
        }

        ctx.set_i32_at(AppContext::RESULT_PHASE, 1)?;
        ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;
        ctx.set_i32_at(AppContext::DECK_BAR_SLIDE, 0x3e8)?;

        return Ok(true);
    }

    if (phase as u32) <= 2 {
        if (frame as u32) < 0xd {
            return Ok(true);
        }

        ctx.set_i32_at(AppContext::RESULT_PHASE, phase.wrapping_add(1))?;
        ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;

        return Ok(true);
    }

    if phase == 7 {
        if touch_is_down(ctx)? != 0 {
            if ctx.u8_at(AppContext::DECK_ROW_SWAPPING)? | ctx.u8_at(AppContext::CAMERA_DRAGGING)?
                != 0
            {
                ctx.set_block_at::<1>(AppContext::TOUCH_CAPTURED, [1])?;
            }
        } else if touch_released(ctx)? == 0 {
            ctx.set_block_at::<1>(AppContext::TOUCH_CAPTURED, [0])?;
        }

        let press = ctx.i32_at(AppContext::RESULT_OK_PRESS)?;

        if press > 0 {
            ctx.set_i32_at(AppContext::RESULT_OK_PRESS, press.wrapping_add(1))?;

            if (press as u32) < 5 {
                return Ok(true);
            }

            ctx.set_i32_at(AppContext::RESULT_OK_PRESS, 0)?;

            let status =
                ranking_status_by_id(&ctx.ranking_entries, ctx.i32_at(AppContext::MAP_INDEX)?);

            if get_map_type(ctx, 0)? == 4 && (status.wrapping_sub(1) as u32) <= 1 {
                ctx.set_i32_at(
                    AppContext::RESULT_PHASE,
                    ctx.i32_at(AppContext::RESULT_PHASE)?.wrapping_add(1),
                )?;
            } else {
                ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1])?;
                ctx.set_i32_at(AppContext::CURTAIN_STYLE, 1)?;
            }

            app_on_draw(ctx)?;

            return Ok(false);
        }

        if ctx.i32_at(AppContext::DECK_ROW_SWAPPING)? | ctx.i32_at(AppContext::SWIPE_VELOCITY)? != 0
        {
            return Ok(true);
        }

        let map = button_bank_find(&ctx.buttons, 0xc8).ok_or(Fault::NullPointer { site: SITE })?;

        new_button_set_touchable(&mut ctx.buttons, map, 0)?;

        let share =
            button_bank_find(&ctx.buttons, 0xc9).ok_or(Fault::NullPointer { site: SITE })?;

        new_button_set_touchable(&mut ctx.buttons, share, 0)?;

        if labyrinth_active(ctx)? {
            let labyrinth =
                button_bank_find(&ctx.buttons, 0xca).ok_or(Fault::NullPointer { site: SITE })?;

            new_button_set_touchable(&mut ctx.buttons, labyrinth, 0)?;
        }

        let hovered = touch_is_down(ctx)? != 0 && {
            let x = ctx.i32_at(AppContext::RESULT_OK_RECT)?;
            let y = ctx.i32_at(AppContext::RESULT_OK_RECT + 4)?;
            let width = ctx.i32_at(AppContext::RESULT_OK_RECT + 8)?;
            let height = ctx.i32_at(AppContext::RESULT_OK_RECT + 0xc)?;

            hit_test_rect(ctx, x, y, width, height)?
        };

        if hovered {
            if ctx.u8_at(AppContext::DECK_BUTTON_PRESSED)? == 0 {
                play_sound(sound_manager(ctx)?, 0xa, None);
                ctx.set_block_at::<1>(AppContext::DECK_BUTTON_PRESSED, [1])?;
            }
        } else {
            ctx.set_block_at::<1>(AppContext::DECK_BUTTON_PRESSED, [0])?;
        }

        let released = touch_released(ctx)? != 0
            && {
                let x = ctx.i32_at(AppContext::RESULT_OK_RECT)?;
                let y = ctx.i32_at(AppContext::RESULT_OK_RECT + 4)?;
                let width = ctx.i32_at(AppContext::RESULT_OK_RECT + 8)?;
                let height = ctx.i32_at(AppContext::RESULT_OK_RECT + 0xc)?;

                hit_test_rect(ctx, x, y, width, height)?
            }
            && !button_bank_busy(&ctx.buttons)?;

        if released || back_pressed(ctx)? != 0 {
            ctx.set_i32_at(
                AppContext::RESULT_OK_PRESS,
                ctx.i32_at(AppContext::RESULT_OK_PRESS)?.wrapping_add(1),
            )?;
            play_sound(sound_manager(ctx)?, 0xb, None);

            return Ok(true);
        }

        if get_map_type(ctx, 0)? == 4 {
            return Ok(true);
        }

        if ctx.i32_at(AppContext::RESULT_MAP_LOCKED)? != 0 {
            return Ok(true);
        }

        if ctx.u8_at(AppContext::CURTAIN_ACTIVE)? != 0 {
            return Ok(true);
        }

        if dialog_top(ctx).is_some() {
            return Ok(true);
        }

        let map = button_bank_find(&ctx.buttons, 0xc8).ok_or(Fault::NullPointer { site: SITE })?;

        new_button_set_touchable(&mut ctx.buttons, map, 1)?;

        let share =
            button_bank_find(&ctx.buttons, 0xc9).ok_or(Fault::NullPointer { site: SITE })?;

        new_button_set_touchable(&mut ctx.buttons, share, 1)?;

        if labyrinth_active(ctx)? {
            let labyrinth =
                button_bank_find(&ctx.buttons, 0xca).ok_or(Fault::NullPointer { site: SITE })?;

            new_button_set_touchable(&mut ctx.buttons, labyrinth, 1)?;
        }

        return Ok(true);
    }

    if phase == 3 {
        ctx.set_i32_at(
            AppContext::RESULT_OK_RECT,
            operation::div_2(get_drawable_width(ctx)?).wrapping_add(-0xbe),
        )?;
        ctx.set_i32_at(AppContext::RESULT_OK_RECT + 4, 0x280)?;
        ctx.set_i32_at(AppContext::RESULT_OK_RECT + 8, 0x17d)?;
        ctx.set_i32_at(AppContext::RESULT_OK_RECT + 0xc, 0x58)?;

        let mut slide = ctx.i32_at(AppContext::RESULT_OK_SLIDE)?.wrapping_add(0x14);

        ctx.set_i32_at(AppContext::RESULT_OK_SLIDE, slide)?;

        let limit = 0x280i32.wrapping_sub(ctx.deck_bar_base_y);

        if slide >= limit {
            ctx.set_i32_at(AppContext::RESULT_OK_SLIDE, limit)?;
            ctx.set_i32_at(AppContext::RESULT_PHASE, 7)?;
            slide = limit;
        }

        ctx.set_i32_at(
            AppContext::RESULT_OK_RECT + 4,
            ctx.i32_at(AppContext::LETTERBOX_SHIFT)?
                .wrapping_sub(slide)
                .wrapping_add(0x278),
        )?;

        return Ok(true);
    }

    if phase <= 0x24 {
        let phase = phase.wrapping_add(1);

        ctx.set_i32_at(AppContext::RESULT_PHASE, phase)?;

        if phase != 0x25 {
            return Ok(true);
        }

        let text = query_localizable(ctx, b"connecting");

        connecting_indicator_show(ctx, &text)?;
        ctx.result_event_sheets[0] = texture_cache_load(
            ctx,
            b"img009_nekoDojo_001.png",
            b"img009_nekoDojo_001.imgcut",
            0x2601,
        )?;
        ctx.result_event_sheets[1] = texture_cache_load(
            ctx,
            b"img009_nekoDojo_result.png",
            b"img009_nekoDojo_result.imgcut",
            0x2601,
        )?;

        if has_inquiry_code(ctx)? {
            let id = ctx.i32_at(AppContext::MAP_INDEX)?;
            let score = ctx.i32_at(AppContext::SCORE_TOTAL)?;

            ranking_submit_score(ctx, id, score)?;
        }

        return Ok(true);
    }

    if phase == 0x25 {
        return Ok(true);
    }

    let next = phase.wrapping_add(1);

    ctx.set_i32_at(AppContext::RESULT_PHASE, next)?;

    if next == 0x44 {
        let map = button_bank_find(&ctx.buttons, 0xc8).ok_or(Fault::NullPointer { site: SITE })?;

        new_button_set_touchable(&mut ctx.buttons, map, 1)?;

        let share =
            button_bank_find(&ctx.buttons, 0xc9).ok_or(Fault::NullPointer { site: SITE })?;

        new_button_set_touchable(&mut ctx.buttons, share, 1)?;

        let pattern = query_localizable(ctx, b"ranking_resultcomment");
        let rank = ranking_rank_by_id(&ctx.ranking_entries, ctx.i32_at(AppContext::MAP_INDEX)?);
        let name = ranking_name_by_id(&ctx.ranking_entries, ctx.i32_at(AppContext::MAP_INDEX)?);
        let name = query_localizable(ctx, &name);
        let message = string_format_rank_comment(ctx, &pattern, rank, &name)?;

        dialog_show_alt(
            ctx,
            &message,
            0,
            0x6e,
            0x140,
            Some(trial_win_update_lambda_1),
        )?;

        return Ok(true);
    }

    if (phase as u32) < 0x43 {
        return Ok(true);
    }

    let hovered = touch_is_down(ctx)? != 0 && {
        let x = ctx.i32_at(AppContext::RESULT_OK_RECT)?;
        let y = ctx.i32_at(AppContext::RESULT_OK_RECT + 4)?;
        let width = ctx.i32_at(AppContext::RESULT_OK_RECT + 8)?;
        let height = ctx.i32_at(AppContext::RESULT_OK_RECT + 0xc)?;

        hit_test_rect(ctx, x, y, width, height)?
    };

    if hovered {
        if ctx.u8_at(AppContext::DECK_BUTTON_PRESSED)? == 0 {
            play_sound(sound_manager(ctx)?, 0xa, None);
            ctx.set_block_at::<1>(AppContext::DECK_BUTTON_PRESSED, [1])?;
        }
    } else {
        ctx.set_block_at::<1>(AppContext::DECK_BUTTON_PRESSED, [0])?;
    }

    if touch_released(ctx)? == 0 {
        return Ok(true);
    }

    let x = ctx.i32_at(AppContext::RESULT_OK_RECT)?;
    let y = ctx.i32_at(AppContext::RESULT_OK_RECT + 4)?;
    let width = ctx.i32_at(AppContext::RESULT_OK_RECT + 8)?;
    let height = ctx.i32_at(AppContext::RESULT_OK_RECT + 0xc)?;

    if !hit_test_rect(ctx, x, y, width, height)? {
        return Ok(true);
    }

    play_sound(sound_manager(ctx)?, 0xb, None);
    ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1])?;
    ctx.set_i32_at(AppContext::CURTAIN_STYLE, 1)?;

    let dialog = dialog_top(ctx).ok_or(Fault::NullPointer { site: SITE })?;

    dialog_close(ctx, dialog)?;

    Ok(true)
}
