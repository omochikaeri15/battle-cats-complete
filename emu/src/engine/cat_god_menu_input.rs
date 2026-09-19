use crate::{operation, Fault};

use super::{
    analytics_record, app_on_draw, back_pressed, bgm_player_switch, call_rng, can_push_back, get_anim_len, get_battle_status, get_design_height2,
    get_drawable_width, get_entity_state, get_global_map_id, get_max_money, get_max_zoom, get_miracle_price, get_stage_index, get_star_level,
    get_text_texture, get_worker_level, hit_test_rect, keep_in_bound, log_analytics_event, obf_value_read, play_sound, save_battle_snapshot,
    set_auto_camera_mode, set_bgm_duck, set_money, set_worker_level, sin_deg, sound_manager, spend_cat_food, text_texture_cache, touch_is_down,
    touch_released, xor_row_get, AppContext, Entity, FormatArg,
};

const SITE: &str = "cat_god_menu_input";

const MIRACLE_ANIM_SLOT: [i32; 4] = [1, 2, 0, 3];

pub fn cat_god_menu_input(ctx: &mut AppContext) -> Result<bool, Fault> {
    let state = ctx.i32_at(AppContext::CAT_GOD_STATE)?;
    let mut chatter = false;

    match state {
        0 => {
            if ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? == 3 {
                ctx.set_i32_at(AppContext::CAT_GOD_INTRO_STEP, 4)?;
            }

            let speed = ctx.f32_at(AppContext::CAT_GOD_SPIN_SPEED)? + 0.5;

            ctx.set_f32_at(AppContext::CAT_GOD_SPIN_SPEED, speed)?;
            ctx.set_f32_at(AppContext::CAT_GOD_SPIN, ctx.f32_at(AppContext::CAT_GOD_SPIN)? + speed)?;
            ctx.set_i32_at(AppContext::CAT_GOD_BUTTON_SINK, ctx.i32_at(AppContext::CAT_GOD_BUTTON_SINK)?.wrapping_sub(1))?;

            if speed >= 30.0 {
                ctx.set_i32_at(AppContext::CAT_GOD_STATE, 1)?;
                ctx.set_i32_at(AppContext::CAT_GOD_BUTTON_SINK, -100)?;

                for slot in ctx.label_texts.iter_mut() {
                    *slot = None;
                }

                ctx.set_i32_at(AppContext::CAT_GOD_CHATTER_TIMER, 0x12c)?;

                ctx.label_texts[0] = {
                    let font = ctx.default_font.clone();
                    let text = ctx.god_intro_texts.first().map(|row| row[0].clone()).ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?;

                    Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                };
                ctx.label_texts[1] = {
                    let font = ctx.default_font.clone();
                    let text = ctx.god_intro_texts.first().map(|row| row[1].clone()).ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?;

                    Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                };
                ctx.label_texts[2] = {
                    let font = ctx.default_font.clone();
                    let text = ctx.god_name_text.clone();

                    Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                };
                ctx.label_texts[6] = {
                    let font = ctx.default_font.clone();
                    let text = ctx.god_intro_texts.get(2).map(|row| row[0].clone()).ok_or(Fault::IndexOutOfRange { site: SITE, index: 2, limit: 3 })?;

                    Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x14, 1, 0))
                };
            }
        }
        1 => {
            let fade = ctx.i32_at(AppContext::CAT_GOD_FADE)?;
            let drop = ctx.i32_at(AppContext::CAT_GOD_DROP_SPEED)?.wrapping_add(0x28);

            ctx.set_i32_at(AppContext::CAT_GOD_DROP_SPEED, drop)?;
            ctx.set_i32_at(AppContext::CAT_GOD_BUTTON_SINK, ctx.i32_at(AppContext::CAT_GOD_BUTTON_SINK)?.wrapping_add(drop))?;
            ctx.set_i32_at(AppContext::CAT_GOD_FADE, fade.wrapping_add(0x14))?;

            if fade >= 0x9f {
                ctx.set_i32_at(AppContext::CAT_GOD_FADE, 0xb2)?;
                ctx.set_i32_at(AppContext::CAT_GOD_STATE, 2)?;
                ctx.set_i32_at(AppContext::CAT_GOD_OFFSET, -600)?;
                ctx.set_i32_at(AppContext::CAT_GOD_SPIN, 0)?;
            }
        }
        2 => {
            let offset = operation::cvttsd2si(ctx.i32_at(AppContext::CAT_GOD_OFFSET)? as f64 * 0.7);

            ctx.set_i32_at(AppContext::CAT_GOD_OFFSET, offset)?;

            if offset >= 0 {
                ctx.set_i32_at(AppContext::CAT_GOD_OFFSET, 0)?;
                ctx.set_i32_at(AppContext::CAT_GOD_STATE, 3)?;
            }
        }
        3 => {
            let spin = ctx.f32_at(AppContext::CAT_GOD_SPIN)? + 6.0;

            ctx.set_f32_at(AppContext::CAT_GOD_SPIN, spin)?;
            ctx.set_i32_at(AppContext::CAT_GOD_BOB, operation::cvttss2si(sin_deg(spin) * 10.0))?;

            let step = ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)?;

            if step == 0 {
                let idle = ctx.i32_at(AppContext::CAT_GOD_IDLE_FRAMES)?;

                ctx.set_i32_at(AppContext::CAT_GOD_IDLE_FRAMES, idle.wrapping_add(1))?;

                if idle >= 0x3b {
                    app_on_draw(ctx)?;
                    ctx.set_i32_at(AppContext::CAT_GOD_IDLE_FRAMES, 0)?;
                    ctx.set_i32_at(AppContext::CAT_GOD_INTRO_STEP, 1)?;
                    ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [1])?;
                    ctx.set_block_at::<16>(AppContext::TUTORIAL_TIMER, [0; 16])?;

                    return Ok(false);
                }
            } else if step >= 2 {
                ctx.set_i32_at(AppContext::CAT_GOD_OPEN_TICKS, ctx.i32_at(AppContext::CAT_GOD_OPEN_TICKS)?.wrapping_add(1))?;
                chatter = true;

                let mut picked = None;

                for miracle in 0..4usize {
                    let at = AppContext::CAT_GOD_PRESSES + miracle * 4;
                    let presses = ctx.i32_at(at)?;

                    if presses > 0 {
                        ctx.set_i32_at(at, presses.wrapping_add(1))?;
                        picked = Some((miracle, presses));
                        break;
                    }
                }

                if let Some((miracle, presses)) = picked {
                    if presses as u32 >= 5 {
                        ctx.set_i32_at(AppContext::CAT_GOD_PRESSES + miracle * 4, 0)?;
                        ctx.set_i32_at(AppContext::CAT_GOD_SELECTED, miracle as i32)?;
                        ctx.set_block_at::<1>(AppContext::CAT_GOD_CONFIRM_OPEN, [1])?;

                        if step as u32 >= 4 {
                            let price = get_miracle_price(ctx, miracle as i32)?;

                            if (obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_16_COUNT)?) as i32) < price {
                                ctx.set_i32_at(AppContext::CAT_GOD_CHATTER_TIMER, 0x12c)?;

                                ctx.label_texts[0] = {
                                    let font = ctx.default_font.clone();
                                    let text = ctx.god_short_texts[0].clone();

                                    Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                                };
                                ctx.label_texts[1] = {
                                    let font = ctx.default_font.clone();
                                    let text = ctx.god_short_texts[1].clone();

                                    Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                                };
                            }
                        }

                        let selected = ctx.i32_at(AppContext::CAT_GOD_SELECTED)? as i64 as usize;

                        ctx.label_texts[3] = {
                            let font = ctx.default_font.clone();
                            let text = ctx.god_item_names.get(selected).cloned().ok_or(Fault::IndexOutOfRange { site: SITE, index: selected as i64, limit: 4 })?;

                            Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                        };

                        let selected = ctx.i32_at(AppContext::CAT_GOD_SELECTED)? as i64 as usize;

                        ctx.label_texts[4] = {
                            let font = ctx.default_font.clone();
                            let text = ctx.god_item_texts.get(selected).map(|row| row[0].clone()).ok_or(Fault::IndexOutOfRange { site: SITE, index: selected as i64, limit: 4 })?;

                            Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                        };

                        let selected = ctx.i32_at(AppContext::CAT_GOD_SELECTED)? as i64 as usize;

                        ctx.label_texts[5] = {
                            let font = ctx.default_font.clone();
                            let text = ctx.god_item_texts.get(selected).map(|row| row[1].clone()).ok_or(Fault::IndexOutOfRange { site: SITE, index: selected as i64, limit: 4 })?;

                            Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                        };
                    }
                } else if ctx.i32_at(AppContext::CAT_GOD_PRESSES + 0x10)? > 0 {
                    let presses = ctx.i32_at(AppContext::CAT_GOD_PRESSES + 0x10)?;

                    ctx.set_i32_at(AppContext::CAT_GOD_PRESSES + 0x10, presses.wrapping_add(1))?;

                    if presses as u32 >= 5 {
                        ctx.set_i32_at(AppContext::CAT_GOD_PRESSES + 0x10, 0)?;
                        ctx.set_block_at::<2>(AppContext::CAT_GOD_MENU_IS_OPEN, [0; 2])?;

                        ctx.set_block_at::<0x144>(AppContext::CAT_GOD_SPIN, [0; 0x144])?;

                        for slot in ctx.label_texts.iter_mut() {
                            *slot = None;
                        }

                        ctx.label_texts[0] = {
                            let font = ctx.default_font.clone();
                            let text = ctx.battle_texts.get(5).cloned().ok_or(Fault::IndexOutOfRange { site: SITE, index: 5, limit: 0x35 })?;

                            Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                        };

                        for label in 0..4usize {
                            ctx.label_texts[1 + label] = {
                                let font = ctx.default_font.clone();
                                let text = ctx.battle_menu_texts.get(4 + label).cloned().ok_or(Fault::IndexOutOfRange { site: SITE, index: (4 + label) as i64, limit: 0x24 })?;

                                Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                            };
                        }

                        for label in 0..3usize {
                            ctx.label_texts[10 + label] = {
                                let font = ctx.default_font.clone();
                                let text = ctx.battle_option_texts.get(3 + label).cloned().ok_or(Fault::IndexOutOfRange { site: SITE, index: (3 + label) as i64, limit: 9 })?;

                                Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                            };
                        }

                        bgm_player_switch(ctx, 0, 1)?;
                        app_on_draw(ctx)?;

                        return Ok(false);
                    }
                } else if ctx.i32_at(AppContext::CAT_GOD_PRESSES + 0x14)? > 0 {
                    let presses = ctx.i32_at(AppContext::CAT_GOD_PRESSES + 0x14)?;

                    ctx.set_i32_at(AppContext::CAT_GOD_PRESSES + 0x14, presses.wrapping_add(1))?;

                    if presses as u32 >= 5 {
                        ctx.set_i32_at(AppContext::CAT_GOD_PRESSES + 0x14, 0)?;

                        if (step as u32) < 4 {
                            ctx.label_texts[0] = {
                                let font = ctx.default_font.clone();
                                let text = ctx.god_bought_texts[0].clone();

                                Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                            };
                            ctx.label_texts[1] = {
                                let font = ctx.default_font.clone();
                                let text = ctx.god_bought_texts[1].clone();

                                Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                            };
                            ctx.set_i32_at(AppContext::CAT_GOD_INTRO_STEP, 3)?;
                        } else {
                            let price = get_miracle_price(ctx, ctx.i32_at(AppContext::CAT_GOD_SELECTED)?)?;

                            if !spend_cat_food(ctx, price)? {
                                app_on_draw(ctx)?;
                                ctx.set_block_at::<1>(AppContext::CAT_FOOD_SHOP_OPEN, [1])?;
                                ctx.set_i32_at(AppContext::CAT_FOOD_SHOP_MODE, 1)?;

                                return Ok(false);
                            }

                            let map = get_global_map_id(ctx, 0)?;
                            let stage = get_stage_index(ctx)?;
                            let star = get_star_level(ctx)?;
                            let price = get_miracle_price(ctx, ctx.i32_at(AppContext::CAT_GOD_SELECTED)?)?;

                            analytics_record(
                                ctx,
                                0x13157fd,
                                price,
                                0,
                                &[
                                    (b"sec1_type", FormatArg::Text(b"MapID")),
                                    (b"sec1_id", FormatArg::Int(map)),
                                    (b"sec2_type", FormatArg::Text(b"StageIdx")),
                                    (b"sec2_id", FormatArg::Int(stage)),
                                    (b"ex_type", FormatArg::Text(b"StageLv")),
                                    (b"ex_id", FormatArg::Int(star)),
                                ],
                            )?;

                            let selected = ctx.i32_at(AppContext::CAT_GOD_SELECTED)?;
                            let chapter = xor_row_get(ctx.bytes_from(AppContext::CHAPTER_PROGRESS)?, 7).ok_or(Fault::IndexOutOfRange { site: SITE, index: 7, limit: 10 })? as i32;

                            log_analytics_event(ctx, 0xb, selected, (chapter >= 0x30) as i32, 0, 0)?;

                            ctx.label_texts[0] = {
                                let font = ctx.default_font.clone();
                                let text = ctx.god_bought_texts[0].clone();

                                Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                            };
                            ctx.label_texts[1] = {
                                let font = ctx.default_font.clone();
                                let text = ctx.god_bought_texts[1].clone();

                                Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                            };
                        }

                        ctx.set_block_at::<1>(AppContext::CAT_GOD_CONFIRM_OPEN, [0])?;
                        ctx.set_i32_at(AppContext::CAT_GOD_STATE, 4)?;
                        save_battle_snapshot(ctx)?;
                    }
                } else if ctx.i32_at(AppContext::CAT_GOD_PRESSES + 0x18)? > 0 {
                    let presses = ctx.i32_at(AppContext::CAT_GOD_PRESSES + 0x18)?;

                    ctx.set_i32_at(AppContext::CAT_GOD_PRESSES + 0x18, presses.wrapping_add(1))?;

                    if presses as u32 >= 5 {
                        ctx.set_i32_at(AppContext::CAT_GOD_PRESSES + 0x18, 0)?;
                        ctx.set_block_at::<1>(AppContext::CAT_GOD_CONFIRM_OPEN, [0])?;

                        if step as u32 >= 4 {
                            let price = get_miracle_price(ctx, ctx.i32_at(AppContext::CAT_GOD_SELECTED)?)?;

                            if (obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_16_COUNT)?) as i32) < price {
                                let line = call_rng(ctx, 0x21) as i64 as usize;

                                ctx.label_texts[0] = {
                                    let font = ctx.default_font.clone();
                                    let text = ctx.god_chatter_texts.get(line).map(|row| row[0].clone()).ok_or(Fault::IndexOutOfRange { site: SITE, index: line as i64, limit: 0x21 })?;

                                    Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                                };
                                ctx.label_texts[1] = {
                                    let font = ctx.default_font.clone();
                                    let text = ctx.god_chatter_texts.get(line).map(|row| row[1].clone()).ok_or(Fault::IndexOutOfRange { site: SITE, index: line as i64, limit: 0x21 })?;

                                    Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                                };
                            }
                        }
                    }
                } else if ctx.i32_at(AppContext::LOSE_SHOP_PRESS)? > 0 {
                    let presses = ctx.i32_at(AppContext::LOSE_SHOP_PRESS)?;

                    ctx.set_i32_at(AppContext::LOSE_SHOP_PRESS, presses.wrapping_add(1))?;

                    if presses as u32 >= 5 {
                        ctx.set_i32_at(AppContext::LOSE_SHOP_PRESS, 0)?;

                        if ctx.i32_at(AppContext::SHOP_TUTORIAL_SEEN)? == 0 {
                            app_on_draw(ctx)?;
                            ctx.set_i32_at(AppContext::SHOP_TUTORIAL_SEEN, 1)?;
                            ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [1])?;
                            ctx.set_block_at::<16>(AppContext::TUTORIAL_TIMER, [0; 16])?;
                            set_bgm_duck(sound_manager(ctx)?, 0x32);

                            return Ok(false);
                        }

                        let food = obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_16_COUNT)?) as i32;

                        app_on_draw(ctx)?;

                        if food < 0xdbba0 {
                            ctx.set_i32_at(AppContext::PENDING_SCENE, 0)?;
                            ctx.set_block_at::<1>(AppContext::SCENE_CHANGE_REQUESTED, [1])?;
                        } else {
                            ctx.set_block_at::<1>(AppContext::CAT_FOOD_SHOP_OPEN, [1])?;
                            ctx.set_i32_at(AppContext::CAT_FOOD_SHOP_MODE, 0x2710)?;
                        }

                        return Ok(false);
                    }
                } else {
                    let mut counter = None;

                    if ctx.u8_at(AppContext::CAT_GOD_CONFIRM_OPEN)? == 0 {
                        if step as u32 >= 4 {
                            for miracle in 0..3usize {
                                let rect = AppContext::CAT_GOD_MIRACLE_RECTS + miracle * 0x10;
                                let held = touch_is_down(ctx)? != 0 && {
                                    let x = ctx.i32_at(rect)?;
                                    let y = ctx.i32_at(rect + 4)?;
                                    let width = ctx.i32_at(rect + 8)?;
                                    let height = ctx.i32_at(rect + 0xc)?;

                                    hit_test_rect(ctx, x, y, width, height)?
                                };

                                if !held {
                                    ctx.set_block_at::<1>(AppContext::CAT_GOD_HOVER + miracle, [0])?;
                                } else if ctx.u8_at(AppContext::CAT_GOD_HOVER + miracle)? == 0 {
                                    play_sound(sound_manager(ctx)?, 0xa, None);
                                    ctx.set_block_at::<1>(AppContext::CAT_GOD_HOVER + miracle, [1])?;
                                }
                            }
                        }

                        if ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? >= 2 {
                            let rect = AppContext::CAT_GOD_MIRACLE_RECTS + 0x30;
                            let held = touch_is_down(ctx)? != 0 && {
                                let x = ctx.i32_at(rect)?;
                                let y = ctx.i32_at(rect + 4)?;
                                let width = ctx.i32_at(rect + 8)?;
                                let height = ctx.i32_at(rect + 0xc)?;

                                hit_test_rect(ctx, x, y, width, height)?
                            };

                            if !held {
                                ctx.set_block_at::<1>(AppContext::CAT_GOD_HOVER + 3, [0])?;
                            } else if ctx.u8_at(AppContext::CAT_GOD_HOVER + 3)? == 0 {
                                play_sound(sound_manager(ctx)?, 0xa, None);
                                ctx.set_block_at::<1>(AppContext::CAT_GOD_HOVER + 3, [1])?;
                            }

                            if ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? >= 4 {
                                for miracle in 0..3usize {
                                    let rect = AppContext::CAT_GOD_MIRACLE_RECTS + miracle * 0x10;
                                    let pressed = touch_released(ctx)? != 0 && {
                                        let x = ctx.i32_at(rect)?;
                                        let y = ctx.i32_at(rect + 4)?;
                                        let width = ctx.i32_at(rect + 8)?;
                                        let height = ctx.i32_at(rect + 0xc)?;

                                        hit_test_rect(ctx, x, y, width, height)?
                                    };

                                    if pressed {
                                        play_sound(sound_manager(ctx)?, 0xb, None);
                                        ctx.set_i32_at(
                                            AppContext::CAT_GOD_PRESSES + miracle * 4,
                                            ctx.i32_at(AppContext::CAT_GOD_PRESSES + miracle * 4)?.wrapping_add(1),
                                        )?;
                                    }
                                }
                            }

                            if ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? >= 2 && touch_released(ctx)? != 0 && {
                                let x = ctx.i32_at(rect)?;
                                let y = ctx.i32_at(rect + 4)?;
                                let width = ctx.i32_at(rect + 8)?;
                                let height = ctx.i32_at(rect + 0xc)?;

                                hit_test_rect(ctx, x, y, width, height)?
                            } {
                                play_sound(sound_manager(ctx)?, 0xb, None);
                                counter = Some(AppContext::CAT_GOD_PRESSES + 0xc);
                            }
                        }
                    } else {
                        let rect = AppContext::CAT_GOD_CONFIRM_RECT;
                        let held = touch_is_down(ctx)? != 0 && {
                            let x = ctx.i32_at(rect)?;
                            let y = ctx.i32_at(rect + 4)?;
                            let width = ctx.i32_at(rect + 8)?;
                            let height = ctx.i32_at(rect + 0xc)?;

                            hit_test_rect(ctx, x, y, width, height)?
                        };

                        if !held {
                            ctx.set_block_at::<1>(AppContext::CAT_GOD_HOVER + 5, [0])?;
                        } else if ctx.u8_at(AppContext::CAT_GOD_HOVER + 5)? == 0 {
                            play_sound(sound_manager(ctx)?, 0xa, None);
                            ctx.set_block_at::<1>(AppContext::CAT_GOD_HOVER + 5, [1])?;
                        }

                        if ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? >= 4 {
                            let rect = AppContext::CAT_GOD_BACK_RECT;
                            let held = touch_is_down(ctx)? != 0 && {
                                let x = ctx.i32_at(rect)?;
                                let y = ctx.i32_at(rect + 4)?;
                                let width = ctx.i32_at(rect + 8)?;
                                let height = ctx.i32_at(rect + 0xc)?;

                                hit_test_rect(ctx, x, y, width, height)?
                            };

                            if !held {
                                ctx.set_block_at::<1>(AppContext::CAT_GOD_HOVER + 6, [0])?;
                            } else if ctx.u8_at(AppContext::CAT_GOD_HOVER + 6)? == 0 {
                                play_sound(sound_manager(ctx)?, 0xa, None);
                                ctx.set_block_at::<1>(AppContext::CAT_GOD_HOVER + 6, [1])?;
                            }
                        }

                        let confirmed = touch_released(ctx)? != 0 && {
                            let x = ctx.i32_at(rect)?;
                            let y = ctx.i32_at(rect + 4)?;
                            let width = ctx.i32_at(rect + 8)?;
                            let height = ctx.i32_at(rect + 0xc)?;

                            hit_test_rect(ctx, x, y, width, height)?
                        };

                        if confirmed {
                            let sound = if ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? < 4 {
                                0xe
                            } else {
                                let price = get_miracle_price(ctx, ctx.i32_at(AppContext::CAT_GOD_SELECTED)?)?;
                                let food = obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_16_COUNT)?) as i32;

                                if food >= price { 0xe } else { 0xf }
                            };

                            play_sound(sound_manager(ctx)?, sound, None);
                            counter = Some(AppContext::CAT_GOD_PRESSES + 0x14);
                        } else if touch_released(ctx)? != 0 {
                            let rect = AppContext::CAT_GOD_BACK_RECT;
                            let x = ctx.i32_at(rect)?;
                            let y = ctx.i32_at(rect + 4)?;
                            let width = ctx.i32_at(rect + 8)?;
                            let height = ctx.i32_at(rect + 0xc)?;
                            let hit = hit_test_rect(ctx, x, y, width, height)?;

                            if hit && ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? >= 4 {
                                play_sound(sound_manager(ctx)?, 0xb, None);
                                counter = Some(AppContext::CAT_GOD_PRESSES + 0x18);
                            }
                        }
                    }

                    if let Some(at) = counter {
                        ctx.set_i32_at(at, ctx.i32_at(at)?.wrapping_add(1))?;
                    }

                    if ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? >= 4 {
                        let rect = AppContext::CAT_GOD_CLOSE_RECT;
                        let held = touch_is_down(ctx)? != 0 && {
                            let x = ctx.i32_at(rect)?;
                            let y = ctx.i32_at(rect + 4)?;
                            let width = ctx.i32_at(rect + 8)?;
                            let height = ctx.i32_at(rect + 0xc)?;

                            hit_test_rect(ctx, x, y, width, height)?
                        };

                        if !held {
                            ctx.set_block_at::<1>(AppContext::CAT_GOD_HOVER + 4, [0])?;
                        } else if ctx.u8_at(AppContext::CAT_GOD_HOVER + 4)? == 0 {
                            play_sound(sound_manager(ctx)?, 0xa, None);
                            ctx.set_block_at::<1>(AppContext::CAT_GOD_HOVER + 4, [1])?;
                        }

                        let shop_held = touch_is_down(ctx)? != 0 && {
                            let x = ctx.i32_at(AppContext::LOSE_SHOP_RECT)?;
                            let y = ctx.i32_at(AppContext::LOSE_SHOP_RECT + 4)?;
                            let width = ctx.i32_at(AppContext::LOSE_SHOP_RECT + 8)?;
                            let height = ctx.i32_at(AppContext::LOSE_SHOP_RECT + 0xc)?;

                            hit_test_rect(ctx, x, y, width, height)?
                        };

                        if !shop_held {
                            ctx.set_block_at::<1>(AppContext::LOSE_SHOP_HELD, [0])?;
                        } else if ctx.u8_at(AppContext::LOSE_SHOP_HELD)? == 0 {
                            play_sound(sound_manager(ctx)?, 0xa, None);
                            ctx.set_block_at::<1>(AppContext::LOSE_SHOP_HELD, [1])?;
                        }

                        let closed = touch_released(ctx)? != 0 && {
                            let x = ctx.i32_at(rect)?;
                            let y = ctx.i32_at(rect + 4)?;
                            let width = ctx.i32_at(rect + 8)?;
                            let height = ctx.i32_at(rect + 0xc)?;

                            hit_test_rect(ctx, x, y, width, height)?
                        };

                        if closed || back_pressed(ctx)? != 0 {
                            play_sound(sound_manager(ctx)?, 0xb, None);
                            ctx.set_i32_at(AppContext::CAT_GOD_PRESSES + 0x10, ctx.i32_at(AppContext::CAT_GOD_PRESSES + 0x10)?.wrapping_add(1))?;
                        }

                        let shop = touch_released(ctx)? != 0 && {
                            let x = ctx.i32_at(AppContext::LOSE_SHOP_RECT)?;
                            let y = ctx.i32_at(AppContext::LOSE_SHOP_RECT + 4)?;
                            let width = ctx.i32_at(AppContext::LOSE_SHOP_RECT + 8)?;
                            let height = ctx.i32_at(AppContext::LOSE_SHOP_RECT + 0xc)?;

                            hit_test_rect(ctx, x, y, width, height)?
                        };

                        if shop {
                            play_sound(sound_manager(ctx)?, 0xb, None);
                            ctx.set_i32_at(AppContext::LOSE_SHOP_PRESS, ctx.i32_at(AppContext::LOSE_SHOP_PRESS)?.wrapping_add(1))?;
                        }
                    }
                }
            }
        }
        4 => {
            let frame = ctx.i32_at(AppContext::CAT_GOD_SETTLE_FRAME)?;

            ctx.set_i32_at(AppContext::CAT_GOD_SETTLE_FRAME, frame.wrapping_add(1))?;

            if frame == 0 {
                ctx.set_i32_at(AppContext::CAT_GOD_OFFSET, 0)?;
                ctx.set_i32_at(AppContext::CAT_GOD_VELOCITY, -2)?;
            } else if frame > 0x31 {
                let velocity = ctx.i32_at(AppContext::CAT_GOD_VELOCITY)?;

                ctx.set_i32_at(AppContext::CAT_GOD_OFFSET, ctx.i32_at(AppContext::CAT_GOD_OFFSET)?.wrapping_add(velocity))?;
                ctx.set_i32_at(AppContext::CAT_GOD_VELOCITY, velocity.wrapping_mul(2))?;
            } else if frame > 0x1c {
                if frame as u32 > 0x26 {
                    let offset = ctx.i32_at(AppContext::CAT_GOD_OFFSET)?;

                    if frame != 0x31 {
                        ctx.set_i32_at(AppContext::CAT_GOD_OFFSET, offset.wrapping_sub(1))?;
                    } else {
                        ctx.set_i32_at(AppContext::CAT_GOD_OFFSET, offset.wrapping_sub(2))?;
                    }
                } else if frame & 1 != 0 {
                    ctx.set_i32_at(AppContext::CAT_GOD_OFFSET, ctx.i32_at(AppContext::CAT_GOD_OFFSET)?.wrapping_sub(1))?;
                }
            } else if frame.wrapping_add(1) % 3 == 0 {
                ctx.set_i32_at(AppContext::CAT_GOD_OFFSET, ctx.i32_at(AppContext::CAT_GOD_OFFSET)?.wrapping_sub(1))?;
            }

            if ctx.i32_at(AppContext::CAT_GOD_OFFSET)? <= -2000 {
                ctx.set_i32_at(AppContext::CAT_GOD_STATE, 5)?;
            }
        }
        _ => {}
    }

    if chatter {
        let due = if ctx.u8_at(AppContext::CAT_GOD_CONFIRM_OPEN)? == 0 || ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? < 4 {
            let timer = ctx.i32_at(AppContext::CAT_GOD_CHATTER_TIMER)?;

            ctx.set_i32_at(AppContext::CAT_GOD_CHATTER_TIMER, timer.wrapping_sub(1))?;
            timer <= 1
        } else {
            let price = get_miracle_price(ctx, ctx.i32_at(AppContext::CAT_GOD_SELECTED)?)?;

            if (obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_16_COUNT)?) as i32) < price {
                false
            } else {
                let timer = ctx.i32_at(AppContext::CAT_GOD_CHATTER_TIMER)?;

                ctx.set_i32_at(AppContext::CAT_GOD_CHATTER_TIMER, timer.wrapping_sub(1))?;
                timer <= 1
            }
        };

        if due {
            let line = call_rng(ctx, 0x21) as i64 as usize;

            ctx.label_texts[0] = {
                let font = ctx.default_font.clone();
                let text = ctx.god_chatter_texts.get(line).map(|row| row[0].clone()).ok_or(Fault::IndexOutOfRange { site: SITE, index: line as i64, limit: 0x21 })?;

                Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
            };
            ctx.label_texts[1] = {
                let font = ctx.default_font.clone();
                let text = ctx.god_chatter_texts.get(line).map(|row| row[1].clone()).ok_or(Fault::IndexOutOfRange { site: SITE, index: line as i64, limit: 0x21 })?;

                Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
            };
            ctx.set_i32_at(AppContext::CAT_GOD_CHATTER_TIMER, 0x12c)?;
        }
    }

    if ctx.i32_at(AppContext::CAT_GOD_STATE)? & !1 == 4 && ctx.i32_at(AppContext::CAT_GOD_ZOOM_DONE)? == 0 {
        let ticks = ctx.i32_at(AppContext::CAT_GOD_ZOOM_TICKS)?;

        ctx.set_i32_at(AppContext::CAT_GOD_ZOOM_TICKS, ticks.wrapping_add(1))?;

        let zoom = ctx.i32_at(AppContext::CAMERA_ZOOM)?;
        let (delta, min_zoom) = if ticks == 0 {
            let camera_x = ctx.i32_at(AppContext::CAMERA_X)?;
            let min_zoom = ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?;
            let delta = min_zoom.wrapping_mul(-100).wrapping_add(zoom);

            ctx.set_i32_at(AppContext::CAT_GOD_ZOOM_DELTA, delta)?;
            ctx.set_i32_at(AppContext::CAT_GOD_SAVED_ZOOM, zoom)?;
            ctx.set_i32_at(AppContext::CAT_GOD_SAVED_CAMERA_X, camera_x)?;

            (delta, min_zoom)
        } else {
            (ctx.i32_at(AppContext::CAT_GOD_ZOOM_DELTA)?, ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?)
        };

        let old_zoom = zoom as f32;
        let scale = operation::cvttss2si(old_zoom / 100.0);

        ctx.set_i32_at(AppContext::SCRATCH_0, scale)?;
        ctx.set_i32_at(AppContext::SCRATCH_1, scale)?;

        let zoomed = operation::div_neg_200(delta).wrapping_add(zoom);

        ctx.set_i32_at(AppContext::CAMERA_ZOOM, zoomed)?;

        let floor = min_zoom.wrapping_mul(100);
        let zoom = if zoomed < floor {
            ctx.set_i32_at(AppContext::CAMERA_ZOOM, floor)?;
            floor
        } else if zoomed <= get_max_zoom(ctx)? {
            ctx.i32_at(AppContext::CAMERA_ZOOM)?
        } else {
            let max = get_max_zoom(ctx)?;

            ctx.set_i32_at(AppContext::CAMERA_ZOOM, max)?;
            max
        };

        let new_zoom = zoom as f32;
        let percent = new_zoom / 100.0;
        let scale = operation::cvttss2si(percent);

        ctx.set_i32_at(AppContext::SCRATCH_0, scale)?;
        ctx.set_i32_at(AppContext::SCRATCH_2, scale)?;

        let camera_x = ctx.i32_at(AppContext::CAMERA_X)? as f64;
        let centered = ((96000000.0f32 / old_zoom) as f64 * 0.5 + camera_x) as f32 as f64;
        let camera_x = operation::cvttsd2si(centered - (96000000.0f32 / new_zoom) as f64 * 0.5);

        ctx.set_i32_at(AppContext::CAMERA_X, camera_x)?;

        let right = -9600.0f32 / (percent / 100.0) + ctx.i32_at(AppContext::STAGE_LENGTH)? as f32;
        let clamped = if camera_x as f32 > right {
            Some(operation::cvttss2si(right))
        } else if camera_x < 0 {
            Some(0)
        } else {
            None
        };

        if let Some(x) = clamped {
            ctx.set_i32_at(AppContext::CAMERA_X, x)?;
            ctx.set_i32_at(AppContext::SWIPE_VELOCITY, 0)?;

            if get_battle_status(ctx)? == 0 {
                set_auto_camera_mode(ctx, 0)?;
            }
        }
    }

    if ctx.i32_at(AppContext::CAT_GOD_STATE)? == 5 {
        let frame = ctx.i32_at(AppContext::CAT_GOD_ANIM_FRAME)?;
        let selected = ctx.i32_at(AppContext::CAT_GOD_SELECTED)?;

        ctx.set_i32_at(AppContext::CAT_GOD_ANIM_FRAME, frame.wrapping_add(1))?;

        let lead = if selected != 0 { 0 } else { -40 };

        if frame == 0 {
            if selected == 0 {
                play_sound(sound_manager(ctx)?, 0x23, None);
            }

            if ctx.i32_at(AppContext::CAT_GOD_SELECTED)? == 1 {
                play_sound(sound_manager(ctx)?, 0x26, None);
            }

            if ctx.i32_at(AppContext::CAT_GOD_SELECTED)? == 2 {
                play_sound(sound_manager(ctx)?, 0x27, None);
            }

            if ctx.i32_at(AppContext::CAT_GOD_SELECTED)? == 3 {
                play_sound(sound_manager(ctx)?, 0x28, None);
            }
        }

        let frame = ctx.i32_at(AppContext::CAT_GOD_ANIM_FRAME)?;
        let selected = ctx.i32_at(AppContext::CAT_GOD_SELECTED)?;
        let slot = *MIRACLE_ANIM_SLOT.get(selected as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: selected as i64, limit: 4 })?;
        let anim = ctx.miracle_anims.get(slot as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: slot as i64, limit: 4 })?;
        let length = get_anim_len(&anim[0])?.wrapping_add(lead);
        let selected = ctx.i32_at(AppContext::CAT_GOD_SELECTED)?;
        let slot = *MIRACLE_ANIM_SLOT.get(selected as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: selected as i64, limit: 4 })?;

        let reached = frame >= length || (slot == 3 && ctx.i32_at(AppContext::CAT_GOD_ANIM_FRAME)? >= 0x17c);

        if !reached {
            ctx.set_i32_at(AppContext::CAT_GOD_TICKS, ctx.i32_at(AppContext::CAT_GOD_TICKS)?.wrapping_add(1))?;

            return Ok(true);
        }

        let anim = ctx.miracle_anims.get(slot as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: slot as i64, limit: 4 })?;

        ctx.set_i32_at(AppContext::CAT_GOD_ANIM_FRAME, get_anim_len(&anim[0])?)?;

        let finished = match ctx.i32_at(AppContext::CAT_GOD_SELECTED)? {
            0 => {
                let length = get_anim_len(&ctx.miracle_anims[1][1])?;

                for bolt in 0..3usize {
                    let at = AppContext::CAT_GOD_FRAMES + bolt * 4;
                    let current = if bolt == 0 {
                        let old = ctx.i32_at(at)?;

                        ctx.set_i32_at(at, old.wrapping_add(1))?;

                        if old == 0 {
                            play_sound(sound_manager(ctx)?, 0x24, None);
                        }

                        ctx.i32_at(at)?
                    } else {
                        let current = ctx.i32_at(at)?;

                        if current == 1 {
                            play_sound(sound_manager(ctx)?, 0x24, None);
                        }

                        ctx.i32_at(at)?
                    };

                    let current = if current == 0x20 {
                        play_sound(sound_manager(ctx)?, 0x25, None);
                        ctx.i32_at(at)?
                    } else {
                        current
                    };

                    if bolt < 2 {
                        if current >= 0x1e {
                            ctx.set_i32_at(at + 4, ctx.i32_at(at + 4)?.wrapping_add(1))?;

                            if current >= length {
                                ctx.set_i32_at(at, length)?;
                            }
                        }
                    } else if current >= length {
                        ctx.set_i32_at(at, length)?;
                    }
                }

                ctx.i32_at(AppContext::CAT_GOD_FRAMES + 8)? >= length
            }
            1 => {
                let old = ctx.i32_at(AppContext::CAT_GOD_FRAMES)?;

                ctx.set_i32_at(AppContext::CAT_GOD_FRAMES, old.wrapping_add(1))?;

                if old == 0 {
                    ctx.set_i32_at(AppContext::CAT_GOD_PUSHING, 1)?;

                    for slot in 1..0x33 {
                        if ctx.i32_at(AppContext::entity_field(1, slot, Entity::OCCUPANT))? != 0
                            && get_entity_state(ctx, 1, slot)? != 4
                            && get_entity_state(ctx, 1, slot)? != 0x15
                        {
                            ctx.set_i32_at(AppContext::entity_field(1, slot, Entity::GOD_PUSH_FRAME), 0)?;
                            ctx.set_i32_at(AppContext::entity_field(1, slot, Entity::GOD_PUSH_STEP), 10)?;

                            let delay = call_rng(ctx, 0x3c).wrapping_add(10);

                            ctx.set_i32_at(AppContext::entity_field(1, slot, Entity::GOD_PUSH_DELAY), delay)?;
                        }
                    }
                }

                let frames = ctx.i32_at(AppContext::CAT_GOD_FRAMES)?;

                if frames > 0 {
                    for slot in 1..0x33 {
                        if ctx.i32_at(AppContext::entity_field(1, slot, Entity::OCCUPANT))? != 0
                            && get_entity_state(ctx, 1, slot)? != 4
                            && get_entity_state(ctx, 1, slot)? != 0x15
                        {
                            let pushed = ctx.i32_at(AppContext::entity_field(1, slot, Entity::GOD_PUSH_FRAME))?;
                            let delay = ctx.i32_at(AppContext::entity_field(1, slot, Entity::GOD_PUSH_DELAY))?;
                            let movable = can_push_back(ctx, 1, slot)?;

                            if pushed >= delay {
                                if movable {
                                    let step = ctx.i32_at(AppContext::entity_field(1, slot, Entity::GOD_PUSH_STEP))?;
                                    let x = AppContext::entity_field(1, slot, Entity::POS_X);

                                    ctx.set_i32_at(x, ctx.i32_at(x)?.wrapping_sub(step))?;
                                    keep_in_bound(ctx, 1, slot)?;
                                }

                                let step = ctx.i32_at(AppContext::entity_field(1, slot, Entity::GOD_PUSH_STEP))?;

                                ctx.set_i32_at(AppContext::entity_field(1, slot, Entity::GOD_PUSH_STEP), if step >= 0xc350 { 0xf4240 } else { step.wrapping_mul(2) })?;
                            } else if movable {
                                let x = AppContext::entity_field(1, slot, Entity::POS_X);

                                ctx.set_i32_at(x, ctx.i32_at(x)?.wrapping_sub(1))?;
                                keep_in_bound(ctx, 1, slot)?;
                            }

                            let at = AppContext::entity_field(1, slot, Entity::GOD_PUSH_FRAME);

                            ctx.set_i32_at(at, ctx.i32_at(at)?.wrapping_add(1))?;
                        }
                    }
                }

                let frames = if frames > 0 { ctx.i32_at(AppContext::CAT_GOD_FRAMES)? } else { frames };
                let length = get_anim_len(&ctx.miracle_anims[2][1])?;

                if frames < length {
                    false
                } else {
                    ctx.set_i32_at(AppContext::CAT_GOD_FRAMES, length)?;

                    if ctx.i32_at(AppContext::CAT_GOD_PUSHING)? == 1 {
                        for slot in 1..0x33 {
                            if ctx.i32_at(AppContext::entity_field(1, slot, Entity::OCCUPANT))? != 0 && get_entity_state(ctx, 1, slot)? != 4 {
                                get_entity_state(ctx, 1, slot)?;
                            }
                        }

                        ctx.set_i32_at(AppContext::CAT_GOD_PUSHING, 0)?;
                    }

                    true
                }
            }
            2 => {
                let old = ctx.i32_at(AppContext::CAT_GOD_FRAMES)?;

                ctx.set_i32_at(AppContext::CAT_GOD_FRAMES, old.wrapping_add(1))?;

                if old == 0 {
                    let x = call_rng(ctx, 0x14).wrapping_sub(10);

                    ctx.set_i32_at(AppContext::CAT_GOD_SHAKE_X, x)?;

                    let y = call_rng(ctx, 0x14).wrapping_sub(5);

                    ctx.set_i32_at(AppContext::CAT_GOD_SHAKE_Y, y)?;
                }

                for wave in 0..9usize {
                    if ctx.i32_at(AppContext::CAT_GOD_FRAMES + wave * 4)? < 6 {
                        continue;
                    }

                    let at = AppContext::CAT_GOD_FRAMES + (wave + 1) * 4;
                    let next = ctx.i32_at(at)?;

                    ctx.set_i32_at(at, next.wrapping_add(1))?;

                    if next != 0 {
                        continue;
                    }

                    loop {
                        let x = call_rng(ctx, 0x14).wrapping_sub(10);

                        ctx.set_i32_at(AppContext::CAT_GOD_SHAKE_X + (wave + 1) * 4, x)?;

                        let y = call_rng(ctx, 0x14).wrapping_sub(5);

                        ctx.set_i32_at(AppContext::CAT_GOD_SHAKE_Y + (wave + 1) * 4, y)?;

                        if ctx.i32_at(AppContext::CAT_GOD_SHAKE_X + wave * 4)? != x || ctx.i32_at(AppContext::CAT_GOD_SHAKE_Y + wave * 4)? != y {
                            break;
                        }
                    }
                }

                let mut done = false;

                for wave in 0..10usize {
                    let at = AppContext::CAT_GOD_FRAMES + wave * 4;
                    let current = ctx.i32_at(at)?;
                    let length = get_anim_len(&ctx.miracle_anims[0][1])?;

                    if current >= length {
                        ctx.set_i32_at(at, get_anim_len(&ctx.miracle_anims[0][1])?)?;
                        done = wave == 9;
                    }
                }

                done
            }
            3 => {
                let frames = ctx.i32_at(AppContext::CAT_GOD_FRAMES)?;
                let frames = if frames == 0 {
                    let width = get_drawable_width(ctx)?;

                    ctx.set_i32_at(AppContext::CAT_GOD_FLASH_X, operation::div_2(width).wrapping_sub(0x56))?;

                    let height = get_design_height2(ctx);

                    ctx.set_i32_at(AppContext::CAT_GOD_BOB, operation::div_2(height).wrapping_sub(0x93))?;
                    ctx.i32_at(AppContext::CAT_GOD_FRAMES)?
                } else {
                    frames
                };

                ctx.set_i32_at(AppContext::CAT_GOD_FRAMES, frames.wrapping_add(10))?;

                if frames < 0xf5 {
                    false
                } else {
                    ctx.set_i32_at(AppContext::CAT_GOD_FRAMES, 0xff)?;

                    let x = ctx.i32_at(AppContext::CAT_GOD_FLASH_X)?;
                    let y = ctx.i32_at(AppContext::CAT_GOD_BOB)?;

                    ctx.set_i32_at(AppContext::CAT_GOD_FLASH_X, operation::div_2(x))?;
                    ctx.set_i32_at(AppContext::CAT_GOD_BOB, operation::div_2(y))?;

                    if x <= 1 && y <= 1 {
                        ctx.set_i32_at(AppContext::BABY_BOOM_FRAMES, 0)?;
                        true
                    } else {
                        false
                    }
                }
            }
            _ => true,
        };

        if finished {
            let fade = ctx.i32_at(AppContext::CAT_GOD_FADE)?;

            ctx.set_i32_at(AppContext::CAT_GOD_FADE, fade.wrapping_sub(0x14))?;
            ctx.set_i32_at(AppContext::CAT_GOD_ZOOM_DONE, 1)?;

            if fade <= 0x14 {
                ctx.set_i32_at(AppContext::CAT_GOD_FADE, 0)?;
            }

            ctx.set_i32_at(AppContext::CAT_GOD_ZOOM_TICKS, 0)?;

            let delta = ctx.i32_at(AppContext::CAT_GOD_ZOOM_DELTA)?;
            let ticks = ctx.i32_at(AppContext::CAT_GOD_RETURN_TICKS)?;

            ctx.set_i32_at(AppContext::CAT_GOD_RETURN_TICKS, ticks.wrapping_add(1))?;

            if ticks == 0 {
                ctx.set_i32_at(AppContext::CAT_GOD_RETURN_STEP, operation::div_5(delta).wrapping_mul(2))?;
            }

            let zoom = ctx.i32_at(AppContext::CAMERA_ZOOM)?;
            let old_zoom = zoom as f32;
            let scale = operation::cvttss2si(old_zoom / 100.0);

            ctx.set_i32_at(AppContext::SCRATCH_0, scale)?;
            ctx.set_i32_at(AppContext::SCRATCH_1, scale)?;

            let zoomed = operation::div_10(delta).wrapping_add(zoom);

            ctx.set_i32_at(AppContext::CAMERA_ZOOM, zoomed)?;

            let floor = ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?.wrapping_mul(100);
            let zoom = if zoomed < floor {
                ctx.set_i32_at(AppContext::CAMERA_ZOOM, floor)?;
                floor
            } else if zoomed <= get_max_zoom(ctx)? {
                ctx.i32_at(AppContext::CAMERA_ZOOM)?
            } else {
                let max = get_max_zoom(ctx)?;

                ctx.set_i32_at(AppContext::CAMERA_ZOOM, max)?;
                max
            };

            let new_zoom = zoom as f32;
            let scale = operation::cvttss2si(new_zoom / 100.0);

            ctx.set_i32_at(AppContext::SCRATCH_0, scale)?;
            ctx.set_i32_at(AppContext::SCRATCH_2, scale)?;

            let camera_x = ctx.i32_at(AppContext::CAMERA_X)? as f64;
            let centered = ((96000000.0f32 / old_zoom) as f64 * 0.5 + camera_x) as f32 as f64;
            let camera_x = operation::cvttsd2si(centered - (96000000.0f32 / new_zoom) as f64 * 0.5);

            ctx.set_i32_at(AppContext::CAMERA_X, camera_x)?;

            let floor = ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?.wrapping_mul(100);
            let zoom = if zoom < floor {
                ctx.set_i32_at(AppContext::CAMERA_ZOOM, floor)?;
                floor
            } else {
                let saved = ctx.i32_at(AppContext::CAT_GOD_SAVED_ZOOM)?;

                if zoom > saved {
                    ctx.set_i32_at(AppContext::CAMERA_ZOOM, saved)?;
                    saved
                } else {
                    zoom
                }
            };

            ctx.set_i32_at(AppContext::SCRATCH_1, operation::div_100(zoom))?;

            let scale = zoom as f32 / 100.0 / 100.0;
            let right = -9600.0f32 / scale + ctx.i32_at(AppContext::STAGE_LENGTH)? as f32;
            let clamped = if camera_x as f32 > right {
                Some(operation::cvttss2si(right))
            } else if camera_x < 0 {
                Some(0)
            } else {
                None
            };

            if let Some(x) = clamped {
                ctx.set_i32_at(AppContext::CAMERA_X, x)?;
                ctx.set_i32_at(AppContext::SWIPE_VELOCITY, 0)?;

                if get_battle_status(ctx)? == 0 {
                    set_auto_camera_mode(ctx, 0)?;
                }
            }

            let ticks = ctx.i32_at(AppContext::CAT_GOD_RETURN_TICKS)?;

            set_bgm_duck(sound_manager(ctx)?, 100i32.wrapping_sub(ticks.wrapping_mul(10)));

            if ctx.i32_at(AppContext::CAT_GOD_RETURN_TICKS)? >= 10 {
                ctx.set_i32_at(AppContext::CAT_GOD_RETURN_TICKS, 10)?;
            }

            if ctx.i32_at(AppContext::CAT_GOD_FADE)? <= 0 && ctx.i32_at(AppContext::CAMERA_ZOOM)? >= ctx.i32_at(AppContext::CAT_GOD_SAVED_ZOOM)? {
                if ctx.i32_at(AppContext::CAT_GOD_SELECTED)? == 3 {
                    if get_worker_level(ctx, AppContext::faction_flags(0))? != 7 {
                        ctx.set_i32_at(AppContext::CPU_PENDING_ACTION, 0)?;
                        set_worker_level(ctx, AppContext::faction_flags(0), 7)?;
                        ctx.set_i32_at(AppContext::WORKER_UPGRADE_FX, 0xe)?;
                    }

                    let max = get_max_money(ctx, AppContext::faction_flags(0))?;

                    set_money(ctx, AppContext::faction_flags(0), max)?;
                }

                ctx.set_block_at::<2>(AppContext::CAT_GOD_MENU_IS_OPEN, [0; 2])?;

                ctx.set_block_at::<0x144>(AppContext::CAT_GOD_SPIN, [0; 0x144])?;

                for slot in ctx.label_texts.iter_mut() {
                    *slot = None;
                }

                for label in 0..4usize {
                    ctx.label_texts[1 + label] = {
                        let font = ctx.default_font.clone();
                        let text = ctx.battle_menu_texts.get(4 + label).cloned().ok_or(Fault::IndexOutOfRange { site: SITE, index: (4 + label) as i64, limit: 0x24 })?;

                        Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                    };
                }

                for label in 0..3usize {
                    ctx.label_texts[10 + label] = {
                        let font = ctx.default_font.clone();
                        let text = ctx.battle_option_texts.get(3 + label).cloned().ok_or(Fault::IndexOutOfRange { site: SITE, index: (3 + label) as i64, limit: 9 })?;

                        Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0))
                    };
                }

                let selected = ctx.i32_at(AppContext::CAT_GOD_SELECTED)?;
                let slot = *MIRACLE_ANIM_SLOT.get(selected as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: selected as i64, limit: 4 })?;

                if slot as u32 <= 3 && slot != 2 {
                    ctx.set_i32_at(AppContext::CAT_GOD_HEAL_PENDING + slot as usize * 4, 1)?;
                }

                bgm_player_switch(ctx, 0, 1)?;
            }
        }
    }

    ctx.set_i32_at(AppContext::CAT_GOD_TICKS, ctx.i32_at(AppContext::CAT_GOD_TICKS)?.wrapping_add(1))?;

    Ok(true)
}
