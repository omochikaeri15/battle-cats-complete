use crate::{Fault, operation};

use super::{
    AppContext, app_on_draw, back_pressed, button_bank_busy, button_bank_find, check_medals,
    dialog_set_back_button, dialog_set_on_update, dialog_show, dialog_show_alt, dialog_top,
    get_auto_camera_mode, get_bottom_inset_logical, get_cat_name, get_drawable_width,
    get_item_name, handle_battle_swipe_pinch, hit_test_rect, is_score_stage, labyrinth_active,
    labyrinth_result_ready, new_button_set_touchable, play_sound, point_lose_update_lambda_0,
    point_lose_update_lambda_1, point_lose_update_lambda_2, query_localizable, record_stage_played,
    request_save_data, reward_def_lookup, sound_manager, substitute_tokens, touch_is_down,
    touch_released, xor_row46_get,
};

const SITE: &str = "point_lose_update";

pub fn point_lose_update(ctx: &mut AppContext) -> Result<bool, Fault> {
    ctx.set_i32_at(AppContext::SPEED, 1)?;

    let mut phase = ctx.i32_at(AppContext::OUTRO_PHASE)?;

    if phase != 0 && phase <= 7 {
        handle_battle_swipe_pinch(ctx)?;
        phase = ctx.i32_at(AppContext::OUTRO_PHASE)?;
    } else {
        ctx.set_i32_at(AppContext::SWIPE_VELOCITY, 0)?;
    }

    let frame = ctx.i32_at(AppContext::OUTRO_FRAME)?;
    let next = frame.wrapping_add(1);

    ctx.set_i32_at(AppContext::OUTRO_FRAME, next)?;
    ctx.set_i32_at(
        AppContext::OUTRO_TICKS,
        ctx.i32_at(AppContext::OUTRO_TICKS)?.wrapping_add(1),
    )?;

    if phase == 0 {
        if frame <= 0 {
            ctx.set_i32_at(AppContext::BATTLE_RESUMED, 0)?;
            ctx.set_i32_at(AppContext::BATTLE_CONTINUED, 0)?;
            record_stage_played(ctx)?;
            request_save_data(ctx)?;
        }

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

        if ctx.i32_at(AppContext::OUTRO_FRAME)? < 0x14 {
            return Ok(true);
        }

        ctx.set_i32_at(AppContext::OUTRO_PHASE, 1)?;
        ctx.set_i32_at(AppContext::OUTRO_FRAME, 0)?;
        ctx.set_i32_at(AppContext::DECK_BAR_SLIDE, 0x3e8)?;

        return Ok(true);
    }

    if (phase as u32) <= 2 {
        if frame >= 0xc {
            ctx.set_i32_at(AppContext::OUTRO_PHASE, phase.wrapping_add(1))?;
            ctx.set_i32_at(AppContext::OUTRO_FRAME, 0)?;

            return Ok(true);
        }

        if phase != 2 || frame > -2 {
            return Ok(true);
        }

        if !is_score_stage(ctx.event_items.as_ref()) {
            return Ok(true);
        }

        if (ctx.i32_at(AppContext::OUTRO_FRAME)?.wrapping_add(0x64) as u32) < 0x2e {
            return Ok(true);
        }

        ctx.set_i32_at(AppContext::OUTRO_FRAME, 0)?;

        return Ok(true);
    }

    match phase {
        3 => {
            let mut frame = next;

            if frame == 0xf {
                if ctx.u8_at(AppContext::POINT_LIMIT_PENDING)? == 0 {
                    return Ok(true);
                }

                ctx.set_block_at::<1>(AppContext::POINT_LIMIT_PENDING, [0])?;

                let text = query_localizable(ctx, b"point_stage_limit");
                let dialog = dialog_show(ctx, &text, 0, 0, 4, Some(point_lose_update_lambda_0))?;
                let dialog = dialog_set_on_update(ctx, dialog, Some(point_lose_update_lambda_1))?;

                dialog_set_back_button(ctx, dialog, 0)?;

                frame = ctx.i32_at(AppContext::OUTRO_FRAME)?;
            }

            if frame < 0x1e {
                return Ok(true);
            }

            let head = match ctx.reward_queue.first() {
                Some(entry) => Some(*entry.first().ok_or(Fault::IndexOutOfRange {
                    site: SITE,
                    index: 0,
                    limit: 0,
                })?),
                None => None,
            };

            if head != Some(0x11) {
                ctx.set_i32_at(AppContext::OUTRO_PHASE, 6)?;
                ctx.set_i32_at(AppContext::OUTRO_FRAME, 0)?;

                return Ok(true);
            }

            ctx.set_i32_at(AppContext::OUTRO_PHASE, 8)?;
            ctx.set_i32_at(AppContext::OUTRO_FRAME, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, 0)?;
            play_sound(sound_manager(ctx)?, 0x1d, None);

            let mut message = Vec::new();
            let entry = ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: 0,
                limit: 0,
            })?;
            let group = *entry.get(1).ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: 1,
                limit: entry.len() as i64,
            })?;
            let id = *entry.get(2).ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: 2,
                limit: entry.len() as i64,
            })?;
            let reward = reward_def_lookup(&ctx.reward_defs, group, id)
                .ok_or(Fault::NullPointer { site: SITE })?
                .clone();

            match reward.kind {
                1 => {
                    let text = query_localizable(ctx, &reward.message);
                    let name = get_cat_name(ctx, reward.target, 0);

                    message = substitute_tokens(ctx, &text, &[(b"charaName", &name)])?;
                }
                0 => {
                    let text = query_localizable(ctx, &reward.message);
                    let name = get_item_name(ctx, reward.target);
                    let amount = reward.amount.to_string().into_bytes();

                    message = substitute_tokens(
                        ctx,
                        &text,
                        &[(b"itemName", &name), (b"itemNum", &amount)],
                    )?;
                }
                _ => {}
            }

            dialog_show_alt(ctx, &message, 0, 0x191, 1, Some(point_lose_update_lambda_2))?;

            let low = {
                let row = AppContext::MAP_STAGE_ROWS.wrapping_add(
                    (ctx.i32_at(AppContext::STAGE_ROW)? as i64 as usize)
                        .wrapping_mul(AppContext::MAP_STAGE_ROW_STRIDE),
                );
                let entry = ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange {
                    site: SITE,
                    index: 0,
                    limit: 0,
                })?;
                let index = *entry.get(1).ok_or(Fault::IndexOutOfRange {
                    site: SITE,
                    index: 1,
                    limit: entry.len() as i64,
                })?;

                xor_row46_get(ctx.bytes_from(row)?, index as i64 as usize).ok_or(
                    Fault::IndexOutOfRange {
                        site: SITE,
                        index: index as i64,
                        limit: 0x2f,
                    },
                )? as i32
            };

            let medal = if low >= 0x3e8 && {
                let row = AppContext::MAP_STAGE_ROWS.wrapping_add(
                    (ctx.i32_at(AppContext::STAGE_ROW)? as i64 as usize)
                        .wrapping_mul(AppContext::MAP_STAGE_ROW_STRIDE),
                );
                let entry = ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange {
                    site: SITE,
                    index: 0,
                    limit: 0,
                })?;
                let index = *entry.get(1).ok_or(Fault::IndexOutOfRange {
                    site: SITE,
                    index: 1,
                    limit: entry.len() as i64,
                })?;

                xor_row46_get(ctx.bytes_from(row)?, index as i64 as usize).ok_or(
                    Fault::IndexOutOfRange {
                        site: SITE,
                        index: index as i64,
                        limit: 0x2f,
                    },
                )? as i32
            } <= ctx.drop_chara_max_1000
            {
                true
            } else {
                ({
                    let row = AppContext::MAP_STAGE_ROWS.wrapping_add(
                        (ctx.i32_at(AppContext::STAGE_ROW)? as i64 as usize)
                            .wrapping_mul(AppContext::MAP_STAGE_ROW_STRIDE),
                    );
                    let entry = ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange {
                        site: SITE,
                        index: 0,
                        limit: 0,
                    })?;
                    let index = *entry.get(1).ok_or(Fault::IndexOutOfRange {
                        site: SITE,
                        index: 1,
                        limit: entry.len() as i64,
                    })?;

                    xor_row46_get(ctx.bytes_from(row)?, index as i64 as usize).ok_or(
                        Fault::IndexOutOfRange {
                            site: SITE,
                            index: index as i64,
                            limit: 0x2f,
                        },
                    )? as i32
                }) >= 0x44c
                    && {
                        let row = AppContext::MAP_STAGE_ROWS.wrapping_add(
                            (ctx.i32_at(AppContext::STAGE_ROW)? as i64 as usize)
                                .wrapping_mul(AppContext::MAP_STAGE_ROW_STRIDE),
                        );
                        let entry = ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange {
                            site: SITE,
                            index: 0,
                            limit: 0,
                        })?;
                        let index = *entry.get(1).ok_or(Fault::IndexOutOfRange {
                            site: SITE,
                            index: 1,
                            limit: entry.len() as i64,
                        })?;

                        xor_row46_get(ctx.bytes_from(row)?, index as i64 as usize).ok_or(
                            Fault::IndexOutOfRange {
                                site: SITE,
                                index: index as i64,
                                limit: 0x2f,
                            },
                        )? as i32
                    } <= ctx.drop_chara_max_1100
            };

            if medal {
                check_medals(ctx, 4)?;
            }

            Ok(true)
        }
        4 => {
            ctx.set_i32_at(AppContext::OUTRO_PHASE, 3)?;
            ctx.set_i32_at(AppContext::OUTRO_FRAME, 0x1e)?;

            Ok(true)
        }
        5 => {
            let counter = ctx.i32_at(AppContext::REWARD_POP_COUNTER)?.wrapping_add(1);

            ctx.set_i32_at(
                AppContext::REWARD_POP_COUNTER,
                if (counter as u32) < 4 { counter } else { 4 },
            )?;

            if (counter as u32) < 4 {
                return Ok(true);
            }

            let hold = ctx.i32_at(AppContext::REWARD_POP_HOLD)?;

            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, hold.wrapping_add(1))?;

            if hold < 0x1d {
                return Ok(true);
            }

            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, 0x1e)?;

            if touch_released(ctx)? == 0 {
                return Ok(true);
            }

            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
            ctx.set_i32_at(AppContext::OUTRO_PHASE, 6)?;
            ctx.set_i32_at(AppContext::OUTRO_FRAME, 0)?;
            play_sound(sound_manager(ctx)?, 0xb, None);

            Ok(true)
        }
        6 => {
            ctx.set_i32_at(
                AppContext::OUTRO_OK_RECT,
                operation::div_2(get_drawable_width(ctx)?).wrapping_add(-0xbe),
            )?;
            ctx.set_i32_at(AppContext::OUTRO_OK_RECT + 4, 0x280)?;
            ctx.set_i32_at(AppContext::OUTRO_OK_RECT + 8, 0x17d)?;
            ctx.set_i32_at(AppContext::OUTRO_OK_RECT + 0xc, 0x58)?;

            let slide = ctx.i32_at(AppContext::OUTRO_OK_SLIDE)?;
            let mut offset = slide.wrapping_add(0x14);

            ctx.set_i32_at(AppContext::OUTRO_OK_SLIDE, offset)?;

            if slide >= 0x3e {
                ctx.set_i32_at(AppContext::OUTRO_OK_SLIDE, 0x52)?;
                ctx.set_i32_at(AppContext::OUTRO_PHASE, 7)?;
                offset = 0x52;
            }

            let letterbox = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
            let inset = get_bottom_inset_logical(ctx)?;

            ctx.set_i32_at(
                AppContext::OUTRO_OK_RECT + 4,
                letterbox
                    .wrapping_sub(inset.wrapping_add(offset))
                    .wrapping_add(0x278),
            )?;

            Ok(true)
        }
        7 => {
            if touch_is_down(ctx)? != 0 {
                if ctx.u8_at(AppContext::DECK_ROW_SWAPPING)?
                    | ctx.u8_at(AppContext::CAMERA_DRAGGING)?
                    != 0
                {
                    ctx.set_block_at::<1>(AppContext::TOUCH_CAPTURED, [1])?;
                }
            } else if touch_released(ctx)? == 0 {
                ctx.set_block_at::<1>(AppContext::TOUCH_CAPTURED, [0])?;
            }

            let press = ctx.i32_at(AppContext::OUTRO_OK_PRESS)?;

            if press > 0 {
                ctx.set_i32_at(AppContext::OUTRO_OK_PRESS, press.wrapping_add(1))?;

                if (press as u32) < 5 {
                    return Ok(true);
                }

                ctx.set_i32_at(AppContext::OUTRO_OK_PRESS, 0)?;
                app_on_draw(ctx)?;
                ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1])?;
                ctx.set_i32_at(AppContext::CURTAIN_STYLE, 1)?;
                ctx.set_i32_at(AppContext::BATTLE_RESUMED, 0)?;
                ctx.set_i32_at(AppContext::BATTLE_CONTINUED, 0)?;
                request_save_data(ctx)?;

                return Ok(false);
            }

            if ctx.i32_at(AppContext::DECK_ROW_SWAPPING)?
                | ctx.i32_at(AppContext::SWIPE_VELOCITY)?
                == 0
            {
                let hovered = touch_is_down(ctx)? != 0 && {
                    let x = ctx.i32_at(AppContext::OUTRO_OK_RECT)?;
                    let y = ctx.i32_at(AppContext::OUTRO_OK_RECT + 4)?;
                    let width = ctx.i32_at(AppContext::OUTRO_OK_RECT + 8)?;
                    let height = ctx.i32_at(AppContext::OUTRO_OK_RECT + 0xc)?;

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
            }

            if ctx.i32_at(AppContext::DECK_ROW_SWAPPING)?
                | ctx.i32_at(AppContext::SWIPE_VELOCITY)?
                != 0
            {
                return Ok(true);
            }

            let map =
                button_bank_find(&ctx.buttons, 0xc8).ok_or(Fault::NullPointer { site: SITE })?;

            new_button_set_touchable(&mut ctx.buttons, map, 0)?;

            let share =
                button_bank_find(&ctx.buttons, 0xc9).ok_or(Fault::NullPointer { site: SITE })?;

            new_button_set_touchable(&mut ctx.buttons, share, 0)?;

            if ctx.u8_at(AppContext::OUTRO_VIDEO_BUTTON)? != 0 {
                let third = button_bank_find(&ctx.buttons, 0xcb)
                    .ok_or(Fault::NullPointer { site: SITE })?;

                new_button_set_touchable(&mut ctx.buttons, third, 0)?;
            }

            if !button_bank_busy(&ctx.buttons)? {
                let released = touch_released(ctx)? != 0 && {
                    let x = ctx.i32_at(AppContext::OUTRO_OK_RECT)?;
                    let y = ctx.i32_at(AppContext::OUTRO_OK_RECT + 4)?;
                    let width = ctx.i32_at(AppContext::OUTRO_OK_RECT + 8)?;
                    let height = ctx.i32_at(AppContext::OUTRO_OK_RECT + 0xc)?;

                    hit_test_rect(ctx, x, y, width, height)?
                };

                if released || back_pressed(ctx)? != 0 {
                    ctx.set_i32_at(
                        AppContext::OUTRO_OK_PRESS,
                        ctx.i32_at(AppContext::OUTRO_OK_PRESS)?.wrapping_add(1),
                    )?;
                    play_sound(sound_manager(ctx)?, 0xb, None);

                    return Ok(true);
                }
            }

            if ctx.u8_at(AppContext::CURTAIN_ACTIVE)? != 0 {
                return Ok(true);
            }

            if dialog_top(ctx).is_some() {
                return Ok(true);
            }

            if labyrinth_active(ctx)? && !labyrinth_result_ready(ctx)? {
                return Ok(true);
            }

            let share =
                button_bank_find(&ctx.buttons, 0xc9).ok_or(Fault::NullPointer { site: SITE })?;

            new_button_set_touchable(&mut ctx.buttons, share, 1)?;

            if ctx.i32_at(AppContext::OUTRO_MAP_LOCKED)? != 0 {
                return Ok(true);
            }

            let map =
                button_bank_find(&ctx.buttons, 0xc8).ok_or(Fault::NullPointer { site: SITE })?;

            new_button_set_touchable(&mut ctx.buttons, map, 1)?;

            Ok(true)
        }
        _ => Ok(true),
    }
}
