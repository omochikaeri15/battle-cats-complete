use std::rc::Rc;

use crate::{operation, Fault};

use super::{
    altar_recompute, app_on_draw, back_pressed, battle_check_login_bonus, bonus_popup_text, button_bank_busy, button_bank_find, call_rng, check_medals,
    connecting_indicator_show, dialog_close, dialog_set_back_button, dialog_set_on_update, dialog_show, dialog_show_alt, dialog_show_kind2, dialog_top,
    drop_popup_text, ex_group_pick, ex_lottery_stage_key, format_localized, format_string2, format_string3, game_win_update_lambda_0,
    game_win_update_lambda_1, game_win_update_lambda_10, game_win_update_lambda_11, game_win_update_lambda_12, game_win_update_lambda_13,
    game_win_update_lambda_14, game_win_update_lambda_15, game_win_update_lambda_16, game_win_update_lambda_17, game_win_update_lambda_19,
    game_win_update_lambda_2, game_win_update_lambda_3, game_win_update_lambda_4, game_win_update_lambda_5, game_win_update_lambda_6,
    game_win_update_lambda_7, game_win_update_lambda_8, game_win_update_lambda_9, get_altar_level_cap, get_auto_camera_mode, get_bottom_inset_logical,
    get_castle_enemy_row, get_cat_name, get_cleared_count, get_drawable_width, get_global_map_id, get_item_name, get_map_index, get_map_type,
    get_stage_index, get_stage_name, get_text_texture, handle_battle_swipe_pinch, has_inquiry_code, hidden_drop_key, hit_test_rect,
    imgcut_get_sprite_cut, is_score_stage, labyrinth_active, labyrinth_rank, labyrinth_result_ready, labyrinth_submit, labyrinth_unit_count,
    map_index_of_map_id, map_type_of_map_id, new_button_register, new_button_set_animated, new_button_set_touchable, play_sound, query_localizable,
    reward_def_lookup, sound_manager, stage_pair_record, string_format_int, string_format_rank_comment, substitute_tokens, text_texture_cache,
    texture_cache_load, touch_is_down, touch_released, ui_node_add_child, ui_node_set_anchor, ui_node_set_sprite, xor_row46_get, AppContext,
    DialogEventHandler, STAGE_DISPLAY_ORDER, ZOMBIE_CLEAR_ROWS,
};

const SITE: &str = "game_win_update";

pub fn game_win_update(ctx: &mut AppContext) -> Result<bool, Fault> {
    ctx.set_i32_at(AppContext::SPEED, 1)?;

    let phase = if ctx.i32_at(AppContext::RESULT_PHASE)? != 0 {
        handle_battle_swipe_pinch(ctx)?;
        ctx.i32_at(AppContext::RESULT_PHASE)?
    } else {
        ctx.set_i32_at(AppContext::SWIPE_VELOCITY, 0)?;
        0
    };

    let frame = ctx.i32_at(AppContext::RESULT_FRAME)?;
    let next = frame.wrapping_add(1);

    ctx.set_i32_at(AppContext::RESULT_FRAME, next)?;
    ctx.set_i32_at(AppContext::RESULT_TICKS, ctx.i32_at(AppContext::RESULT_TICKS)?.wrapping_add(1))?;

    if phase == 0 {
        ctx.set_i32_at(AppContext::CAMERA_ZOOM, ctx.i32_at(AppContext::CAMERA_ZOOM)?.wrapping_add(0x320))?;
        ctx.set_i32_at(AppContext::DECK_BAR_SLIDE, ctx.i32_at(AppContext::DECK_BAR_SLIDE)?.wrapping_add(0xa))?;

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
        if frame >= 0xc {
            let phase = phase.wrapping_add(1);

            ctx.set_i32_at(AppContext::RESULT_PHASE, phase)?;
            ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;

            if phase == 2 && is_score_stage(ctx.event_items.as_ref()) {
                ctx.set_i32_at(AppContext::RESULT_FRAME, -100)?;
            }

            return Ok(true);
        }

        if phase != 2 || frame > -2 || !is_score_stage(ctx.event_items.as_ref()) {
            return Ok(true);
        }

        if (ctx.i32_at(AppContext::RESULT_FRAME)?.wrapping_add(0x64) as u32) < 0x2e {
            return Ok(true);
        }

        ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;

        return Ok(true);
    }

    if phase == 3 {
        if ctx.i32_at(AppContext::FIRST_STAGE_WON)? | ctx.i32_at(AppContext::STAGE_ROW)? == 0 {
            ctx.set_i32_at(AppContext::FIRST_STAGE_WON, 1)?;
        }

        let mut frame = next;

        if next == 0xf {
            if ctx.u8_at(AppContext::POINT_LIMIT_PENDING)? == 0 {
                return Ok(true);
            }

            ctx.set_block_at::<1>(AppContext::POINT_LIMIT_PENDING, [0])?;

            let text = query_localizable(ctx, b"point_stage_limit");
            let dialog = dialog_show(ctx, &text, 0, 0, 4, Some(game_win_update_lambda_0))?;
            let dialog = dialog_set_on_update(ctx, dialog, Some(game_win_update_lambda_1))?;

            dialog_set_back_button(ctx, dialog, 0)?;
            frame = ctx.i32_at(AppContext::RESULT_FRAME)?;
        }

        if frame < 0x1e {
            return Ok(true);
        }

        if ctx.reward_queue.len() as i32 <= 0 {
            if ctx.i32_at(AppContext::NEXT_STAGE_UNLOCKED)? == -1 {
                ctx.set_i32_at(AppContext::RESULT_PHASE, 6)?;
                ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;

                return Ok(true);
            }

            ctx.set_i32_at(AppContext::RESULT_PHASE, 5)?;
            ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;
            play_sound(sound_manager(ctx)?, 0x1d, None);

            let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;

            if mode == 3 || mode == 0x63 {
                return Ok(true);
            }

            for slot in ctx.label_texts.iter_mut() {
                *slot = None;
            }

            let unlocked = ctx.i32_at(AppContext::NEXT_STAGE_UNLOCKED)?;

            ctx.label_texts[0] = {
                let font = ctx.default_font.clone();
                let text = ctx.next_stage_names
                    .get(mode as i64 as usize)
                    .and_then(|names| names.get(unlocked as i64 as usize))
                    .map(|names| names[0].clone())
                    .ok_or(Fault::IndexOutOfRange { site: SITE, index: unlocked as i64, limit: 0 })?;

                Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 0, 0))
            };
            ctx.label_texts[1] = {
                let font = ctx.default_font.clone();
                let text = ctx.next_stage_caption.clone();

                Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 0, 0))
            };

            return Ok(true);
        }

        let kind = *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })? ;

        if kind as u32 > 0x11 {
            return Ok(true);
        }

        let mut message: Vec<u8>;
        let handler: DialogEventHandler;

        match kind {
            0 => {
            ctx.set_i32_at(AppContext::RESULT_PHASE, 8)?;
            ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, 0)?;
            play_sound(sound_manager(ctx)?, 0x1d, None);
                let item = *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })? ;
                let first = ((*ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(2).ok_or(Fault::IndexOutOfRange { site: SITE, index: 2, limit: 0 })?) == 1) as u8;
                let amount = *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(3).ok_or(Fault::IndexOutOfRange { site: SITE, index: 3, limit: 0 })? ;

                message = drop_popup_text(ctx, item, first, amount)?;
                dialog_show_alt(ctx, &message, 0, 0x191, 1, Some(game_win_update_lambda_2))?;

            let low = {
                let row = AppContext::MAP_STAGE_ROWS.wrapping_add((ctx.i32_at(AppContext::STAGE_ROW)? as i64 as usize).wrapping_mul(AppContext::MAP_STAGE_ROW_STRIDE));
                let index = *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })? ;

                xor_row46_get(ctx.bytes_from(row)?, index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: index as i64, limit: 0x2f })? as i32
            };

            let medal = if low >= 0x3e8 && {
                let row = AppContext::MAP_STAGE_ROWS.wrapping_add((ctx.i32_at(AppContext::STAGE_ROW)? as i64 as usize).wrapping_mul(AppContext::MAP_STAGE_ROW_STRIDE));
                let index = *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })? ;

                xor_row46_get(ctx.bytes_from(row)?, index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: index as i64, limit: 0x2f })? as i32
            } <= ctx.drop_chara_max_1000 {
                true
            } else {
                ({
                let row = AppContext::MAP_STAGE_ROWS.wrapping_add((ctx.i32_at(AppContext::STAGE_ROW)? as i64 as usize).wrapping_mul(AppContext::MAP_STAGE_ROW_STRIDE));
                let index = *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })? ;

                xor_row46_get(ctx.bytes_from(row)?, index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: index as i64, limit: 0x2f })? as i32
            }) >= 0x44c && {
                let row = AppContext::MAP_STAGE_ROWS.wrapping_add((ctx.i32_at(AppContext::STAGE_ROW)? as i64 as usize).wrapping_mul(AppContext::MAP_STAGE_ROW_STRIDE));
                let index = *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })? ;

                xor_row46_get(ctx.bytes_from(row)?, index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: index as i64, limit: 0x2f })? as i32
            } <= ctx.drop_chara_max_1100
            };

            if medal {
                check_medals(ctx, 4)?;
            }

                return Ok(true);
            }
            1 => {
            ctx.set_i32_at(AppContext::RESULT_PHASE, 8)?;
            ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, 0)?;
            play_sound(sound_manager(ctx)?, 0x1d, None);
                let text = query_localizable(ctx, b"drop_popup_treasure");
                let treasure = ctx.i32_at(AppContext::WIN_TREASURE)? as i64 as usize;
                let first = ctx.treasure_names.get(treasure).cloned().ok_or(Fault::IndexOutOfRange { site: SITE, index: treasure as i64, limit: ctx.treasure_names.len() as i64 })?;
                let map = (ctx.i32_at(AppContext::CHAPTER_MODE)? as i64).wrapping_add(3) as usize;
                let stage = ctx.i32_at(AppContext::CASTLE_ID)? as i64 as usize;
                let second = ctx.stage_names.get(map).and_then(|names| names.get(stage)).cloned().ok_or(Fault::IndexOutOfRange { site: SITE, index: stage as i64, limit: 0 })?;

                message = format_string2(ctx, &text, &first, &second)?;
                handler = game_win_update_lambda_3;
            }
            2 => {
            ctx.set_i32_at(AppContext::RESULT_PHASE, 8)?;
            ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, 0)?;
            play_sound(sound_manager(ctx)?, 0x1d, None);
                ctx.set_block_at::<1>(AppContext::RANK_POPUP_SHOWN, [1])?;

                let item = ctx.i32_at(AppContext::RANK_REWARD_BASE)?.wrapping_add(6);

                message = drop_popup_text(ctx, item, 0, 1)?;
                handler = game_win_update_lambda_4;
            }
            3..=5 => {
            ctx.set_i32_at(AppContext::RESULT_PHASE, 8)?;
            ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, 0)?;
            play_sound(sound_manager(ctx)?, 0x1d, None);
                message = {
                    let key = string_format_int(ctx, b"zombie_clear%02d", kind.wrapping_sub(3))?;

                    query_localizable(ctx, &key)
                };

                let mut energy = false;

                if kind == 5 {
                    if (*ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })?) > 2 {
                        energy = true;
                    } else {
                        let row = (*ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })?) as i64 as usize;
                        let index = *ZOMBIE_CLEAR_ROWS.get(row).ok_or(Fault::IndexOutOfRange { site: SITE, index: row as i64, limit: 3 })?;
                        let key = string_format_int(ctx, b"zombie_clear%02d", index)?;

                        message = query_localizable(ctx, &key);
                    }
                }

                let kind = *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })? ;

                if energy || kind == 5 {
                    let fifty = 0x32.to_string().into_bytes();

                    message = substitute_tokens(ctx, &message, &[(b"EnergyNum", &fifty)])?;
                } else if kind == 4 {
                    if (*ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })?) == 0 {
                        let area = *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(2).ok_or(Fault::IndexOutOfRange { site: SITE, index: 2, limit: 0 })? ;
                        let first = {
                            let key = string_format_int(ctx, b"zombie_area_%02d", area)?;

                            query_localizable(ctx, &key)
                        };
                        let area = *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(2).ok_or(Fault::IndexOutOfRange { site: SITE, index: 2, limit: 0 })? ;
                        let second = {
                            let key = string_format_int(ctx, b"zombie_area_%02d", area)?;

                            query_localizable(ctx, &key)
                        };

                        message = format_string2(ctx, &message, &first, &second)?;
                    } else {
                        message = query_localizable(ctx, b"zombie_clear03");
                    }
                } else if kind == 3 {
                    let map = (*ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })?) as i64 as usize;
                    let order = (*ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(2).ok_or(Fault::IndexOutOfRange { site: SITE, index: 2, limit: 0 })?) as i64 as usize;
                    let stage = *STAGE_DISPLAY_ORDER.get(order).ok_or(Fault::IndexOutOfRange { site: SITE, index: order as i64, limit: 0x33 })? as i64 as usize;
                    let name = ctx.stage_names.get(map).and_then(|names| names.get(stage)).cloned().ok_or(Fault::IndexOutOfRange { site: SITE, index: stage as i64, limit: 0 })?;

                    message = substitute_tokens(ctx, &message, &[(b"stageName", &name)])?;
                }

                handler = game_win_update_lambda_5;
            }
            6 => {
            ctx.set_i32_at(AppContext::RESULT_PHASE, 8)?;
            ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, 0)?;
            play_sound(sound_manager(ctx)?, 0x1d, None);
                message = bonus_popup_text(ctx)?;
                handler = game_win_update_lambda_7;
            }
            7 => {
            ctx.set_i32_at(AppContext::RESULT_PHASE, 8)?;
            ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, 0)?;
            play_sound(sound_manager(ctx)?, 0x1d, None);
                message = query_localizable(ctx, b"Space09_Invasion_stageclear_txt1");
                handler = game_win_update_lambda_6;
            }
            8 => {
            ctx.set_i32_at(AppContext::RESULT_PHASE, 8)?;
            ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, 0)?;
            play_sound(sound_manager(ctx)?, 0x1d, None);
                let text = query_localizable(ctx, b"drop_popup_hidden");
                let key = hidden_drop_key(ctx, *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })? );
                let name = query_localizable(ctx, &key);

                message = format_localized(ctx, &text, &name)?;
                handler = game_win_update_lambda_8;
            }
            9 => {
            ctx.set_i32_at(AppContext::RESULT_PHASE, 8)?;
            ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, 0)?;
            play_sound(sound_manager(ctx)?, 0x1d, None);
                altar_recompute(ctx)?;

                let text = query_localizable(ctx, b"sealed_release");
                let row = get_castle_enemy_row(ctx)?.wrapping_add(-2);
                let level = get_altar_level_cap(ctx, row)?.wrapping_add(1);

                message = string_format_int(ctx, &text, level)?;
                handler = game_win_update_lambda_9;
            }
            10 => {
            ctx.set_i32_at(AppContext::RESULT_PHASE, 8)?;
            ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, 0)?;
            play_sound(sound_manager(ctx)?, 0x1d, None);
                let record = stage_pair_record(ctx, *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })? , *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(2).ok_or(Fault::IndexOutOfRange { site: SITE, index: 2, limit: 0 })? )?.ok_or(Fault::NullPointer { site: SITE })?;
                let map_type = map_type_of_map_id(*ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })? );
                let map_index = map_index_of_map_id(*ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })? );
                let text = query_localizable(ctx, &record.message);
                let stage = *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(2).ok_or(Fault::IndexOutOfRange { site: SITE, index: 2, limit: 0 })? ;
                let first = get_stage_name(ctx, map_type, map_index, stage)?;
                let second = get_stage_name(ctx, map_type, map_index, record.other_stage)?;

                message = substitute_tokens(ctx, &text, &[(b"stageName1", &first), (b"stageName2", &second)])?;
                handler = game_win_update_lambda_10;
            }
            11 => {
            ctx.set_i32_at(AppContext::RESULT_PHASE, 8)?;
            ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, 0)?;
            play_sound(sound_manager(ctx)?, 0x1d, None);
                let food = ctx.item_possession.entry(0x16).or_insert(0);

                *food = food.wrapping_add(0x1e);

                let ticket = ctx.item_possession.entry(0x69).or_insert(0);

                *ticket = ticket.wrapping_add(1);
                message = query_localizable(ctx, b"battle_result_first_clear_bonus");
                handler = game_win_update_lambda_14;
            }
            12 => {
            ctx.set_i32_at(AppContext::RESULT_PHASE, 8)?;
            ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, 0)?;
            play_sound(sound_manager(ctx)?, 0x1d, None);
                let reward = {
                    let text = query_localizable(ctx, b"backstage_result1");
                    let floor = (*ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })?).wrapping_add(1);

                    string_format_int(ctx, &text, floor)?
                };
                let item = *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(2).ok_or(Fault::IndexOutOfRange { site: SITE, index: 2, limit: 0 })? ;
                let mut possession = Vec::new();

                if item & !0x10 != 6 {
                    let text = query_localizable(ctx, b"item_possession");
                    let held = *ctx.item_possession.entry(item).or_insert(0);
                    let first = held.to_string().into_bytes();
                    let held = *ctx.item_possession.entry(item).or_insert(0);
                    let second = held.wrapping_add(*ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(3).ok_or(Fault::IndexOutOfRange { site: SITE, index: 3, limit: 0 })? ).to_string().into_bytes();

                    possession = substitute_tokens(ctx, &text, &[(b"itemNum1", &first), (b"itemNum2", &second)])?;

                    let amount = *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(3).ok_or(Fault::IndexOutOfRange { site: SITE, index: 3, limit: 0 })? ;
                    let held = ctx.item_possession.entry(item).or_insert(0);

                    *held = held.wrapping_add(amount);
                }

                let key: &[u8] = if item == 6 || item == 0xf7 || item == 0xcf { b"drop_popup_xp" } else { b"drop_popup_item" };
                let text = query_localizable(ctx, key);
                let name = get_item_name(ctx, item);
                let amount = (*ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(3).ok_or(Fault::IndexOutOfRange { site: SITE, index: 3, limit: 0 })?).to_string().into_bytes();

                message = substitute_tokens(
                    ctx,
                    &text,
                    &[(b"rewardName", &reward), (b"itemName", &name), (b"itemNum", &amount), (b"itemPossession", &possession)],
                )?;
                handler = game_win_update_lambda_11;
            }
            13 | 14 | 16 => {
            ctx.set_i32_at(AppContext::RESULT_PHASE, 8)?;
            ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, 0)?;
                message = match kind {
                    16 => query_localizable(ctx, b"backstage_clear_limit"),
                    14 => query_localizable(ctx, b"backstage_clear"),
                    13 => query_localizable(ctx, b"backstage_result2"),
                    _ => Vec::new(),
                };

                dialog_show(ctx, &message, 0, 0, 4, Some(game_win_update_lambda_12))?;

                return Ok(true);
            }
            15 => {
            ctx.set_i32_at(AppContext::RESULT_PHASE, 8)?;
            ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, 0)?;
                message = query_localizable(ctx, b"backstage_clear");

                dialog_show_alt(ctx, &message, 0, 0, 4, Some(game_win_update_lambda_13))?;

                return Ok(true);
            }
            _ => {
            ctx.set_i32_at(AppContext::RESULT_PHASE, 8)?;
            ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
            ctx.set_i32_at(AppContext::REWARD_POP_HOLD, 0)?;
            play_sound(sound_manager(ctx)?, 0x1d, None);
                message = Vec::new();

                let reward = reward_def_lookup(&ctx.reward_defs, *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })? , *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(2).ok_or(Fault::IndexOutOfRange { site: SITE, index: 2, limit: 0 })? ).ok_or(Fault::NullPointer { site: SITE })?.clone();

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

                        message = substitute_tokens(ctx, &text, &[(b"itemName", &name), (b"itemNum", &amount)])?;
                    }
                    _ => {}
                }

                dialog_show_alt(ctx, &message, 0, 0x191, 1, Some(game_win_update_lambda_15))?;

            let low = {
                let row = AppContext::MAP_STAGE_ROWS.wrapping_add((ctx.i32_at(AppContext::STAGE_ROW)? as i64 as usize).wrapping_mul(AppContext::MAP_STAGE_ROW_STRIDE));
                let index = *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })? ;

                xor_row46_get(ctx.bytes_from(row)?, index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: index as i64, limit: 0x2f })? as i32
            };

            let medal = if low >= 0x3e8 && {
                let row = AppContext::MAP_STAGE_ROWS.wrapping_add((ctx.i32_at(AppContext::STAGE_ROW)? as i64 as usize).wrapping_mul(AppContext::MAP_STAGE_ROW_STRIDE));
                let index = *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })? ;

                xor_row46_get(ctx.bytes_from(row)?, index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: index as i64, limit: 0x2f })? as i32
            } <= ctx.drop_chara_max_1000 {
                true
            } else {
                ({
                let row = AppContext::MAP_STAGE_ROWS.wrapping_add((ctx.i32_at(AppContext::STAGE_ROW)? as i64 as usize).wrapping_mul(AppContext::MAP_STAGE_ROW_STRIDE));
                let index = *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })? ;

                xor_row46_get(ctx.bytes_from(row)?, index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: index as i64, limit: 0x2f })? as i32
            }) >= 0x44c && {
                let row = AppContext::MAP_STAGE_ROWS.wrapping_add((ctx.i32_at(AppContext::STAGE_ROW)? as i64 as usize).wrapping_mul(AppContext::MAP_STAGE_ROW_STRIDE));
                let index = *ctx.reward_queue.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: 0 })? ;

                xor_row46_get(ctx.bytes_from(row)?, index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: index as i64, limit: 0x2f })? as i32
            } <= ctx.drop_chara_max_1100
            };

            if medal {
                check_medals(ctx, 4)?;
            }

                return Ok(true);
            }
        }

        dialog_show_alt(ctx, &message, 0, 0x191, 1, Some(handler))?;

        return Ok(true);
    }

    if phase == 4 {
        ctx.set_i32_at(AppContext::RESULT_PHASE, 3)?;
        ctx.set_i32_at(AppContext::RESULT_FRAME, 0x1e)?;

        return Ok(true);
    }

    if phase == 5 {
        let counter = ctx.i32_at(AppContext::REWARD_POP_COUNTER)?.wrapping_add(1);

        ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, if (counter as u32) < 4 { counter } else { 4 })?;

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
        ctx.set_i32_at(AppContext::RESULT_PHASE, 6)?;
        ctx.set_i32_at(AppContext::RESULT_FRAME, 0)?;
        play_sound(sound_manager(ctx)?, 0xb, None);

        return Ok(true);
    }

    if phase == 6 {
        ctx.set_i32_at(AppContext::RESULT_OK_RECT, operation::div_2(get_drawable_width(ctx)?).wrapping_add(-0xbe))?;
        ctx.set_i32_at(AppContext::RESULT_OK_RECT + 4, 0x280)?;
        ctx.set_i32_at(AppContext::RESULT_OK_RECT + 8, 0x17d)?;
        ctx.set_i32_at(AppContext::RESULT_OK_RECT + 0xc, 0x58)?;

        let slide = ctx.i32_at(AppContext::RESULT_OK_SLIDE)?;

        ctx.set_i32_at(AppContext::RESULT_OK_SLIDE, slide.wrapping_add(0x14))?;

        'offer: {
            if slide < 0x3e {
                break 'offer;
            }

            ctx.set_i32_at(AppContext::RESULT_OK_SLIDE, 0x52)?;
            ctx.set_i32_at(AppContext::RESULT_PHASE, 7)?;
            ctx.set_block_at::<1>(AppContext::EX_ACCEPTED, [0])?;
            ctx.set_block_at::<2>(AppContext::EX_OFFERED, [0, 0])?;

            let map = get_global_map_id(ctx, 0)?;
            let stage = get_stage_index(ctx)?;
            let pick = ex_group_pick(ctx, map, stage)?;

            if call_rng(ctx, 100) < ctx.i32_at(AppContext::STAGE_EX_CHANCE)? {
                ctx.set_block_at::<1>(AppContext::EX_ROLLED, [1])?;

                let low = ctx.i32_at(AppContext::STAGE_EX_STAGE_MIN)?;

                ctx.set_i32_at(AppContext::EX_MAP, ctx.i32_at(AppContext::STAGE_EX_MAP)?)?;

                let span = ctx.i32_at(AppContext::STAGE_EX_STAGE_MAX)?.wrapping_sub(low).wrapping_add(1);
                let stage = call_rng(ctx, span).wrapping_add(low);

                ctx.set_i32_at(AppContext::EX_STAGE, stage)?;
            } else if pick != -1 {
                ctx.set_block_at::<1>(AppContext::EX_ROLLED, [1])?;

                let key = ex_lottery_stage_key(ctx, pick)?;
                let map = key / 100;

                ctx.set_i32_at(AppContext::EX_MAP, map_index_of_map_id(map))?;
                ctx.set_i32_at(AppContext::EX_STAGE, key.wrapping_sub(map.wrapping_mul(100)))?;
            } else {
                break 'offer;
            }

            ctx.set_block_at::<1>(AppContext::EX_OFFERED, [1])?;

            let text = query_localizable(ctx, b"exstage_text1");
            let map = ctx.i32_at(AppContext::EX_MAP)? as i64 as usize;
            let stage = ctx.i32_at(AppContext::EX_STAGE)? as i64 as usize;
            let name = ctx.ex_stage_names.get(map).and_then(|names| names.get(stage)).cloned().ok_or(Fault::IndexOutOfRange { site: SITE, index: stage as i64, limit: 0 })?;
            let first = format_localized(ctx, &text, &name)?;
            let second = query_localizable(ctx, b"exstage_text2");
            let third = query_localizable(ctx, b"exstage_text3");
            let message = format_string3(ctx, b"%@<br>%@<br>%@", &first, &second, &third)?;
            let dialog = dialog_show_kind2(ctx, &message, 0, 0, 4, Some(game_win_update_lambda_16))?;

            dialog_set_back_button(ctx, dialog, 1)?;
        }

        let slide = ctx.i32_at(AppContext::RESULT_OK_SLIDE)?;
        let letterbox = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
        let inset = get_bottom_inset_logical(ctx)?;

        ctx.set_i32_at(AppContext::RESULT_OK_RECT + 4, letterbox.wrapping_sub(inset.wrapping_add(slide)).wrapping_add(0x278))?;

        return Ok(true);
    }

    if phase == 7 {
        if touch_is_down(ctx)? != 0 {
            if ctx.u8_at(AppContext::DECK_ROW_SWAPPING)? | ctx.u8_at(AppContext::CAMERA_DRAGGING)? != 0 {
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
            app_on_draw(ctx)?;

            if get_map_type(ctx, 0)? == -19 {
                battle_check_login_bonus(ctx)?;

                return Ok(false);
            }

            if labyrinth_active(ctx)? && !labyrinth_result_ready(ctx)? {
                ctx.set_i32_at(AppContext::RESULT_PHASE, 9)?;

                return Ok(false);
            }

            ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1])?;
            ctx.set_i32_at(AppContext::CURTAIN_STYLE, 1)?;

            return Ok(false);
        }

        if ctx.i32_at(AppContext::DECK_ROW_SWAPPING)? | ctx.i32_at(AppContext::SWIPE_VELOCITY)? == 0 {
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
        }

        if ctx.i32_at(AppContext::DECK_ROW_SWAPPING)? | ctx.i32_at(AppContext::SWIPE_VELOCITY)? != 0 {
            return Ok(true);
        }

            {
                let button = button_bank_find(&ctx.buttons, 0xc8).ok_or(Fault::NullPointer { site: SITE })?;

                new_button_set_touchable(&mut ctx.buttons, button, 0)?;
            }
            {
                let button = button_bank_find(&ctx.buttons, 0xc9).ok_or(Fault::NullPointer { site: SITE })?;

                new_button_set_touchable(&mut ctx.buttons, button, 0)?;
            }

        if ctx.u8_at(AppContext::RESULT_VIDEO_BUTTON)? != 0 {
            {
                let button = button_bank_find(&ctx.buttons, 0xcb).ok_or(Fault::NullPointer { site: SITE })?;

                new_button_set_touchable(&mut ctx.buttons, button, 0)?;
            }
        }

        if !button_bank_busy(&ctx.buttons)? {
            let released = touch_released(ctx)? != 0 && {
                let x = ctx.i32_at(AppContext::RESULT_OK_RECT)?;
                let y = ctx.i32_at(AppContext::RESULT_OK_RECT + 4)?;
                let width = ctx.i32_at(AppContext::RESULT_OK_RECT + 8)?;
                let height = ctx.i32_at(AppContext::RESULT_OK_RECT + 0xc)?;

                hit_test_rect(ctx, x, y, width, height)?
            };

            if released || back_pressed(ctx)? != 0 {
                ctx.set_i32_at(AppContext::RESULT_OK_PRESS, ctx.i32_at(AppContext::RESULT_OK_PRESS)?.wrapping_add(1))?;
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

            {
                let button = button_bank_find(&ctx.buttons, 0xc9).ok_or(Fault::NullPointer { site: SITE })?;

                new_button_set_touchable(&mut ctx.buttons, button, 1)?;
            }

        if ctx.i32_at(AppContext::RESULT_MAP_LOCKED)? == 0 {
            {
                let button = button_bank_find(&ctx.buttons, 0xc8).ok_or(Fault::NullPointer { site: SITE })?;

                new_button_set_touchable(&mut ctx.buttons, button, 1)?;
            }
        }

        if ctx.u8_at(AppContext::RESULT_VIDEO_BUTTON)? != 0 {
            {
                let button = button_bank_find(&ctx.buttons, 0xcb).ok_or(Fault::NullPointer { site: SITE })?;

                new_button_set_touchable(&mut ctx.buttons, button, 1)?;
            }
        }

        return Ok(true);
    }

    if (phase.wrapping_sub(9) as u32) <= 0x1b && labyrinth_active(ctx)? {
        let phase = ctx.i32_at(AppContext::RESULT_PHASE)?.wrapping_add(1);

        ctx.set_i32_at(AppContext::RESULT_PHASE, phase)?;

        if phase != 0x25 {
            return Ok(true);
        }

        let text = query_localizable(ctx, b"connecting");

        connecting_indicator_show(ctx, &text)?;
        ctx.result_event_sheets[0] = texture_cache_load(ctx, b"img009_Labyrinth_001.png", b"img009_Labyrinth_001.imgcut", 0x2601)?;
        ctx.result_event_sheets[1] = texture_cache_load(ctx, b"img009_Labyrinth_result.png", b"img009_Labyrinth_result.imgcut", 0x2601)?;

        let map = get_map_index(ctx, 0)?;
        let language = query_localizable(ctx, b"lang");
        let name = string_format_rank_comment(ctx, b"mapname%03d_l_%@.png", map, &language)?;

        ctx.result_event_sheets[2] = texture_cache_load(ctx, &name, b"", 0x2601)?;

        let sheet = Rc::clone(ctx.result_event_sheets[0].as_ref().ok_or(Fault::NullPointer { site: SITE })?);
        let width = imgcut_get_sprite_cut(&sheet, 0x25)?[2];
        let height = imgcut_get_sprite_cut(&sheet, 0x27)?[3];
        let left = operation::div_2(get_drawable_width(ctx)?).wrapping_sub(operation::div_2(width.wrapping_add(0x20)));

        let mut node = ui_node_set_sprite(&sheet, left, 0x12, 0x27)?;

        ui_node_set_anchor(&mut node, 3);

        let child = ui_node_set_sprite(&sheet, 0x20, 1, 0x25)?;

        ui_node_set_anchor(ui_node_add_child(&mut node, child), 3);

        let button = new_button_register(
            &mut ctx.buttons,
            0xca,
            left.wrapping_add(-10),
            8,
            width.wrapping_add(0x34),
            height.wrapping_add(0x14),
            Some(node),
            Some(game_win_update_lambda_17),
        );
        let button = new_button_set_touchable(&mut ctx.buttons, button, 0)?;

        new_button_set_animated(&mut ctx.buttons, button, 0)?;

        if has_inquiry_code(ctx)? {
            let cleared = get_cleared_count(ctx, AppContext::LABYRINTH)?;
            let units = labyrinth_unit_count(ctx, -1)?;

            labyrinth_submit(ctx, cleared, units)?;
        }

        return Ok(true);
    }

    let phase = ctx.i32_at(AppContext::RESULT_PHASE)?;

    if phase < 0x26 {
        return Ok(true);
    }

    if !labyrinth_active(ctx)? {
        return Ok(true);
    }

    let old = ctx.i32_at(AppContext::RESULT_PHASE)?;

    ctx.set_i32_at(AppContext::RESULT_PHASE, old.wrapping_add(1))?;

    if old.wrapping_add(1) == 0x27 {
        play_sound(sound_manager(ctx)?, 0x2a, None);

        let text = query_localizable(ctx, b"labyrinth_result");
        let rank = labyrinth_rank(ctx)?.to_string().into_bytes();
        let map = get_map_index(ctx, 0)?;
        let stage = get_stage_index(ctx)?;
        let name = get_stage_name(ctx, -21, map, stage)?;
        let message = substitute_tokens(ctx, &text, &[(b"rankingNum", &rank), (b"stageName", &name)])?;

        dialog_show_alt(ctx, &message, 0, 0x6e, 0x140, Some(game_win_update_lambda_19))?;

        return Ok(true);
    }

    if old < 0x43 {
        return Ok(true);
    }

            {
                let button = button_bank_find(&ctx.buttons, 0xc8).ok_or(Fault::NullPointer { site: SITE })?;

                new_button_set_touchable(&mut ctx.buttons, button, 1)?;
            }
            {
                let button = button_bank_find(&ctx.buttons, 0xc9).ok_or(Fault::NullPointer { site: SITE })?;

                new_button_set_touchable(&mut ctx.buttons, button, 1)?;
            }
            {
                let button = button_bank_find(&ctx.buttons, 0xca).ok_or(Fault::NullPointer { site: SITE })?;

                new_button_set_touchable(&mut ctx.buttons, button, 1)?;
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

    let released = touch_released(ctx)? != 0 && {
                let x = ctx.i32_at(AppContext::RESULT_OK_RECT)?;
                let y = ctx.i32_at(AppContext::RESULT_OK_RECT + 4)?;
                let width = ctx.i32_at(AppContext::RESULT_OK_RECT + 8)?;
                let height = ctx.i32_at(AppContext::RESULT_OK_RECT + 0xc)?;

                hit_test_rect(ctx, x, y, width, height)?
            };

    if !released && back_pressed(ctx)? == 0 {
        return Ok(true);
    }

    play_sound(sound_manager(ctx)?, 0xb, None);
    ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1])?;
    ctx.set_i32_at(AppContext::CURTAIN_STYLE, 1)?;

    let dialog = dialog_top(ctx).ok_or(Fault::NullPointer { site: SITE })?;

    dialog_close(ctx, dialog)?;

            {
                let button = button_bank_find(&ctx.buttons, 0xc8).ok_or(Fault::NullPointer { site: SITE })?;

                new_button_set_touchable(&mut ctx.buttons, button, 0)?;
            }
            {
                let button = button_bank_find(&ctx.buttons, 0xc9).ok_or(Fault::NullPointer { site: SITE })?;

                new_button_set_touchable(&mut ctx.buttons, button, 0)?;
            }
            {
                let button = button_bank_find(&ctx.buttons, 0xca).ok_or(Fault::NullPointer { site: SITE })?;

                new_button_set_touchable(&mut ctx.buttons, button, 0)?;
            }

    Ok(true)
}
