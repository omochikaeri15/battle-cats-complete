use std::rc::Rc;

use crate::{Fault, ops};

use super::{
    AppContext, Entity, ads_available, app_on_draw, back_pressed, battle_check_login_bonus,
    battle_continue, bc_log_defeated, button_bank_busy, button_bank_find, dialog_show_alt,
    dialog_top, feature_enabled, game_lose_update_lambda_0, game_lose_update_lambda_1,
    get_auto_camera_mode, get_design_height2, get_drawable_width, get_global_map_id, get_hp,
    get_stage_record, get_text_texture, hit_test_rect, imgcut_get_sprite_cut, is_boss,
    lose_exit_map_check, new_button_register, new_button_set_touchable, now_seconds,
    obf_value_read, pick_lose_tip, play_sound, query_localizable, record_stage_played,
    request_save_data, reward_ad_ready, server_config_int, set_bgm_duck, sound_manager,
    std_string_append, string_format_boss_hp, string_format_boss_hp_line, text_texture_cache,
    touch_is_down, touch_released, ui_node_set_anchor, ui_node_set_sprite, web_popup_clear,
    web_popup_open, web_popup_pending,
};

pub fn game_lose_update(ctx: &mut AppContext) -> Result<bool, Fault> {
    ctx.set_i32_at(AppContext::SPEED, 1)?;

    let phase = ctx.i32_at(AppContext::OUTRO_PHASE)?;
    let frame = ctx.i32_at(AppContext::OUTRO_FRAME)?;

    ctx.set_i32_at(AppContext::OUTRO_FRAME, frame.wrapping_add(1))?;

    if phase as u32 > 4 {
        return Ok(true);
    }

    match phase {
        0 => {
            if frame <= 0 {
                ctx.set_i32_at(AppContext::BATTLE_RESUMED, 0)?;
                ctx.set_i32_at(AppContext::BATTLE_CONTINUED, 0)?;
                record_stage_played(ctx)?;
                request_save_data(ctx)?;
            }

            ctx.set_i32_at(
                AppContext::DECK_BAR_SLIDE,
                ctx.i32_at(AppContext::DECK_BAR_SLIDE)?.wrapping_add(0xa),
            )?;
            ctx.set_i32_at(
                AppContext::CAMERA_ZOOM,
                ctx.i32_at(AppContext::CAMERA_ZOOM)?.wrapping_add(0x320),
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

            Ok(true)
        }
        1 => {
            if (frame.wrapping_add(1) as u32) < 0x2c {
                return Ok(true);
            }

            ctx.set_i32_at(AppContext::OUTRO_PHASE, 2)?;
            ctx.set_i32_at(AppContext::OUTRO_FRAME, 0)?;

            Ok(true)
        }
        2 => {
            if frame < 0x4f {
                return Ok(true);
            }

            let continues = ctx.i32_at(AppContext::TUTORIAL_CLEARED)? != 0
                && get_stage_record(ctx, -2, 0, 2, 0, 0)? != 0;

            if !continues {
                ctx.set_i32_at(AppContext::OUTRO_PHASE, 4)?;
                ctx.set_i32_at(AppContext::OUTRO_FRAME, 0)?;

                for label in 0..4usize {
                    let text = ctx
                        .warning2_rows
                        .get(2)
                        .map(|row| row[label].clone())
                        .ok_or(Fault::index_out_of_range(2, 0))?;
                    let font = ctx.default_font.clone();

                    ctx.label_texts[label] = Some(get_text_texture(
                        text_texture_cache(ctx)?,
                        &text,
                        &font,
                        0x1e,
                        1,
                        0,
                    ));
                }

                return Ok(true);
            }

            if ctx.i32_at(AppContext::OUTRO_FRAME)? == 0x50 {
                if web_popup_pending(ctx, 4) {
                    web_popup_clear(ctx, 4);

                    let map = get_global_map_id(ctx, 0)?;
                    let stage = ctx.i32_at(AppContext::STAGE_ROW)?;

                    web_popup_open(ctx, 4, map, stage)?;
                    ctx.set_i32_at(
                        AppContext::OUTRO_FRAME,
                        ctx.i32_at(AppContext::OUTRO_FRAME)?.wrapping_sub(1),
                    )?;
                } else if ctx.i32_at(AppContext::PENDING_SCENE)? == 0x66 {
                    ctx.set_i32_at(
                        AppContext::OUTRO_FRAME,
                        ctx.i32_at(AppContext::OUTRO_FRAME)?.wrapping_sub(1),
                    )?;
                }

                app_on_draw(ctx)?;

                return Ok(false);
            }

            let banner = ctx.i32_at(AppContext::LOSE_BANNER_Y)?;

            ctx.set_i32_at(AppContext::LOSE_BANNER_Y, banner.wrapping_sub(0xa))?;

            if banner > -0x5a {
                return Ok(true);
            }

            ctx.set_i32_at(AppContext::LOSE_BANNER_Y, -0x64)?;
            ctx.set_i32_at(AppContext::OUTRO_PHASE, 3)?;
            ctx.set_i32_at(AppContext::OUTRO_FRAME, 0)?;

            let mut found = false;
            let mut percent = 0i32;

            for slot in 0..0x33 {
                if !is_boss(ctx, 1, slot)? {
                    continue;
                }

                if get_hp(ctx, 1, slot)? <= 0 {
                    continue;
                }

                let max = ctx.i32_at(AppContext::entity_field(1, slot, Entity::MAX_HP))? as f32;
                let hp = ctx.i32_at(AppContext::entity_field(1, slot, Entity::HP))? as f32;
                let value = ops::cvttss2si(hp / max * 100.0);

                percent = if value == 0 { 1 } else { value };
                found = true;

                break;
            }

            if ctx.i32_at(AppContext::STAGE_NO_CONTINUES)? == 1 {
                let mut message = if ctx.u8_at(AppContext::LEADERSHIP_REFUND)? != 0 {
                    query_localizable(ctx, b"leadershipreturn_pop2")
                } else {
                    ctx.warning2_rows
                        .get(0x62)
                        .map(|row| row[0].clone())
                        .ok_or(Fault::index_out_of_range(0x62, 0))?
                };

                if percent != 0 {
                    let label = query_localizable(ctx, b"boss_hp");
                    let line = string_format_boss_hp(ctx, b"<br>%@%d%@", &label, percent, b"%")?;

                    std_string_append(&mut message, &line);
                }

                ctx.set_i32_at(AppContext::OUTRO_PHASE, 5)?;

                let height = get_design_height2(ctx);

                dialog_show_alt(
                    ctx,
                    &message,
                    0,
                    ops::div_4(height),
                    0,
                    Some(game_lose_update_lambda_0),
                )?;

                return Ok(true);
            }

            for label in 0..4usize {
                let font = ctx.default_font.clone();

                if found && label == 2 {
                    let line = ctx.warning2_rows.get(2).map(|row| row[2].clone()).ok_or(
                        Fault::index_out_of_range(2, 0),
                    )?;
                    let boss = query_localizable(ctx, b"boss_hp");
                    let text = string_format_boss_hp_line(
                        ctx,
                        b"%@%@%@%d%@",
                        &line,
                        b"(",
                        &boss,
                        percent,
                        b"%)",
                    )?;

                    ctx.label_texts[2] = Some(get_text_texture(
                        text_texture_cache(ctx)?,
                        &text,
                        &font,
                        0x1e,
                        1,
                        0,
                    ));
                } else {
                    let text = ctx
                        .warning2_rows
                        .get(2)
                        .map(|row| row[label].clone())
                        .ok_or(Fault::index_out_of_range(2, 0))?;

                    ctx.label_texts[label] = Some(get_text_texture(
                        text_texture_cache(ctx)?,
                        &text,
                        &font,
                        0x1e,
                        1,
                        0,
                    ));
                }
            }

            if !ads_available(ctx)? {
                return Ok(true);
            }

            now_seconds(ctx)?;

            if !feature_enabled(ctx, 0x6d)? {
                return Ok(true);
            }

            let now = now_seconds(ctx)?;
            let last = f64::from_le_bytes(ctx.block_at::<8>(AppContext::LAST_VIDEO_TIME)?);
            let hours = server_config_int(ctx, b"CnfContinueAdRewrite", 1, 0x18)?;

            if now < hours.wrapping_mul(0xe10) as f64 + last {
                return Ok(true);
            }

            if !reward_ad_ready(ctx, 1, 0)? {
                return Ok(true);
            }

            ctx.set_block_at::<1>(AppContext::OUTRO_VIDEO_BUTTON, [1])?;

            let sheet = Rc::clone(
                ctx.img004_sheet
                    .as_ref()
                    .ok_or(Fault::null_pointer())?,
            );
            let width = imgcut_get_sprite_cut(&sheet, 0xa)?[2];
            let height = imgcut_get_sprite_cut(&sheet, 0xa)?[3];
            let left = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0));
            let top = 0x15ci32.wrapping_sub(height);

            let mut node = ui_node_set_sprite(
                &sheet,
                left.wrapping_add(ops::div_2(width))
                    .wrapping_add(0x8c),
                ops::div_2(height).wrapping_add(top),
                0xa,
            )?;

            ui_node_set_anchor(&mut node, 1);

            let video = new_button_register(
                &mut ctx.buttons,
                0xcb,
                left.wrapping_add(0x8c),
                top,
                width,
                height,
                Some(node),
                Some(Rc::new(game_lose_update_lambda_1)),
            );

            new_button_set_touchable(&mut ctx.buttons, video, 0)?;

            Ok(true)
        }
        3 => {
            let shown = ctx.i32_at(AppContext::REWARD_POP_COUNTER)?;
            let mut counter = shown.wrapping_add(1);

            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, counter)?;

            if shown == 0 && ctx.i32_at(AppContext::TUTORIAL_CLEARED)? > 0 {
                play_sound(sound_manager(ctx)?, 0x2a, None);
                counter = ctx.i32_at(AppContext::REWARD_POP_COUNTER)?;
            }

            if counter == 1 {
                let half = ops::div_2(get_drawable_width(ctx)?);

                ctx.set_i32_at(AppContext::CANNON_RECT, half.wrapping_add(-0xe5))?;
                ctx.set_i32_at(AppContext::CANNON_RECT + 4, 0x1d8)?;
                ctx.set_i32_at(AppContext::CANNON_RECT + 8, 0xa8)?;
                ctx.set_i32_at(AppContext::CANNON_RECT + 0xc, 0x59)?;

                let half = ops::div_2(get_drawable_width(ctx)?);

                ctx.set_i32_at(AppContext::WORKER_RECT, half.wrapping_add(0x3d))?;
                ctx.set_i32_at(AppContext::WORKER_RECT + 4, 0x1d8)?;
                ctx.set_i32_at(AppContext::WORKER_RECT + 8, 0xa8)?;
                ctx.set_i32_at(AppContext::WORKER_RECT + 0xc, 0x59)?;

                counter = ctx.i32_at(AppContext::REWARD_POP_COUNTER)?;
            }

            if counter as u32 >= 5 {
                ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 4)?;
            } else if counter != 4 {
                return Ok(true);
            }

            if ctx.u8_at(AppContext::OUTRO_VIDEO_BUTTON)? != 0 {
                let video = button_bank_find(&ctx.buttons, 0xcb)
                    .ok_or(Fault::null_pointer())?;

                new_button_set_touchable(&mut ctx.buttons, video, 0)?;
            }

            let yes = ctx.i32_at(AppContext::OUTRO_OK_PRESS)?;

            if yes > 0 {
                ctx.set_i32_at(AppContext::OUTRO_OK_PRESS, yes.wrapping_add(1))?;

                if (yes as u32) < 5 {
                    return Ok(true);
                }

                ctx.set_i32_at(AppContext::OUTRO_OK_PRESS, 0)?;

                if obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_16_COUNT)?) as i32 > 0x1d
                    || ctx.u8_at(AppContext::OUTRO_VIDEO_WATCHED)? != 0
                {
                    battle_continue(ctx)?;

                    return Ok(false);
                }

                app_on_draw(ctx)?;
                ctx.set_block_at::<1>(AppContext::CAT_FOOD_SHOP_OPEN, [1])?;
                ctx.set_i32_at(AppContext::CAT_FOOD_SHOP_MODE, 1)?;

                return Ok(false);
            }

            let no = ctx.i32_at(AppContext::LOSE_NO_PRESS)?;

            if no > 0 {
                ctx.set_i32_at(AppContext::LOSE_NO_PRESS, no.wrapping_add(1))?;

                if (no as u32) < 5 {
                    return Ok(true);
                }

                ctx.set_i32_at(AppContext::LOSE_NO_PRESS, 0)?;
                ctx.set_i32_at(AppContext::OUTRO_PHASE, 4)?;
                ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;

                if ctx.i32_at(AppContext::CHAPTER_MODE)? != 0
                    && ctx.i32_at(AppContext::STAGE_ROW)? != 0
                    && ctx.i32_at(AppContext::LOSE_RECORDED)? == 0
                {
                    ctx.set_i32_at(AppContext::LOSE_RECORDED, 1)?;
                }

                ctx.set_i32_at(AppContext::LOSE_TIP_SHOWN, 1)?;

                let tip = pick_lose_tip(ctx)?;

                ctx.set_i32_at(AppContext::LOSE_TIP, tip)?;

                let mut line = 0i64;

                loop {
                    let tip = ctx.i32_at(AppContext::LOSE_TIP)? as i64 as usize;
                    let row = ctx.lose_rows.get(tip).ok_or(Fault::index_out_of_range(tip as i64, ctx.lose_rows.len() as i64))?;

                    if line >= row.len() as i32 as i64 {
                        break;
                    }

                    let text = row[line as usize].clone();
                    let font = ctx.default_font.clone();
                    let texture =
                        get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0);
                    let slot = 2 + line as usize;

                    *ctx.label_texts
                        .get_mut(slot)
                        .ok_or(Fault::index_out_of_range(slot as i64, 0x434))? = Some(texture);

                    line += 1;
                }

                bc_log_defeated(ctx, 0)?;
                ctx.set_block_at::<1>(AppContext::EX_OFFERED, [0])?;
                ctx.set_i32_at(AppContext::BATTLE_RESUMED, 0)?;
                ctx.set_i32_at(AppContext::BATTLE_CONTINUED, 0)?;
                request_save_data(ctx)?;

                return Ok(true);
            }

            let shop = ctx.i32_at(AppContext::LOSE_SHOP_PRESS)?;

            if shop > 0 {
                ctx.set_i32_at(AppContext::LOSE_SHOP_PRESS, shop.wrapping_add(1))?;

                if (shop as u32) < 5 {
                    return Ok(true);
                }

                ctx.set_i32_at(AppContext::LOSE_SHOP_PRESS, 0)?;

                if ctx.i32_at(AppContext::SHOP_TUTORIAL_SEEN)? == 0 {
                    app_on_draw(ctx)?;
                    ctx.set_i32_at(AppContext::SHOP_TUTORIAL_SEEN, 1)?;
                    ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [1])?;
                    ctx.set_i32_at(AppContext::TUTORIAL_TIMER, 0)?;
                    set_bgm_duck(sound_manager(ctx)?, 0x32);

                    return Ok(false);
                }

                let food = obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_16_COUNT)?) as i32;

                app_on_draw(ctx)?;

                if food >= 0xdbba0 {
                    ctx.set_block_at::<1>(AppContext::CAT_FOOD_SHOP_OPEN, [1])?;
                    ctx.set_i32_at(AppContext::CAT_FOOD_SHOP_MODE, 0x2710)?;
                } else {
                    ctx.set_i32_at(AppContext::PENDING_SCENE, 0)?;
                    ctx.set_block_at::<1>(AppContext::SCENE_CHANGE_REQUESTED, [1])?;
                }

                return Ok(false);
            }

            if button_bank_busy(&ctx.buttons)? {
                return Ok(true);
            }

            if ctx.u8_at(AppContext::OUTRO_VIDEO_BUTTON)? != 0 {
                let video = button_bank_find(&ctx.buttons, 0xcb)
                    .ok_or(Fault::null_pointer())?;

                new_button_set_touchable(&mut ctx.buttons, video, 1)?;
            }

            let yes_hit = touch_is_down(ctx)? != 0 && {
                let x = ctx.i32_at(AppContext::CANNON_RECT)?;
                let y = ctx.i32_at(AppContext::CANNON_RECT + 4)?;
                let width = ctx.i32_at(AppContext::CANNON_RECT + 8)?;
                let height = ctx.i32_at(AppContext::CANNON_RECT + 0xc)?;

                hit_test_rect(ctx, x, y, width, height)?
            };

            if yes_hit {
                ctx.set_i32_at(AppContext::LOSE_CHOICE, 0)?;
            } else if touch_is_down(ctx)? != 0 && {
                let x = ctx.i32_at(AppContext::WORKER_RECT)?;
                let y = ctx.i32_at(AppContext::WORKER_RECT + 4)?;
                let width = ctx.i32_at(AppContext::WORKER_RECT + 8)?;
                let height = ctx.i32_at(AppContext::WORKER_RECT + 0xc)?;

                hit_test_rect(ctx, x, y, width, height)?
            } {
                ctx.set_i32_at(AppContext::LOSE_CHOICE, 1)?;
            }

            let yes_held = touch_is_down(ctx)? != 0 && {
                let x = ctx.i32_at(AppContext::CANNON_RECT)?;
                let y = ctx.i32_at(AppContext::CANNON_RECT + 4)?;
                let width = ctx.i32_at(AppContext::CANNON_RECT + 8)?;
                let height = ctx.i32_at(AppContext::CANNON_RECT + 0xc)?;

                hit_test_rect(ctx, x, y, width, height)?
            };

            if yes_held {
                if ctx.u8_at(AppContext::CANNON_HELD)? == 0 {
                    play_sound(sound_manager(ctx)?, 0xa, None);
                    ctx.set_block_at::<1>(AppContext::CANNON_HELD, [1])?;
                }
            } else {
                ctx.set_block_at::<1>(AppContext::CANNON_HELD, [0])?;
            }

            let no_held = touch_is_down(ctx)? != 0 && {
                let x = ctx.i32_at(AppContext::WORKER_RECT)?;
                let y = ctx.i32_at(AppContext::WORKER_RECT + 4)?;
                let width = ctx.i32_at(AppContext::WORKER_RECT + 8)?;
                let height = ctx.i32_at(AppContext::WORKER_RECT + 0xc)?;

                hit_test_rect(ctx, x, y, width, height)?
            };

            if no_held {
                if ctx.u8_at(AppContext::WORKER_HELD)? == 0 {
                    play_sound(sound_manager(ctx)?, 0xa, None);
                    ctx.set_block_at::<1>(AppContext::WORKER_HELD, [1])?;
                }
            } else {
                ctx.set_block_at::<1>(AppContext::WORKER_HELD, [0])?;
            }

            let yes_released = touch_released(ctx)? != 0 && {
                let x = ctx.i32_at(AppContext::CANNON_RECT)?;
                let y = ctx.i32_at(AppContext::CANNON_RECT + 4)?;
                let width = ctx.i32_at(AppContext::CANNON_RECT + 8)?;
                let height = ctx.i32_at(AppContext::CANNON_RECT + 0xc)?;

                hit_test_rect(ctx, x, y, width, height)?
            };

            if yes_released {
                ctx.set_i32_at(
                    AppContext::OUTRO_OK_PRESS,
                    ctx.i32_at(AppContext::OUTRO_OK_PRESS)?.wrapping_add(1),
                )?;
                play_sound(sound_manager(ctx)?, 0xb, None);
            } else {
                let no_released = touch_released(ctx)? != 0 && {
                    let x = ctx.i32_at(AppContext::WORKER_RECT)?;
                    let y = ctx.i32_at(AppContext::WORKER_RECT + 4)?;
                    let width = ctx.i32_at(AppContext::WORKER_RECT + 8)?;
                    let height = ctx.i32_at(AppContext::WORKER_RECT + 0xc)?;

                    hit_test_rect(ctx, x, y, width, height)?
                };

                if no_released || back_pressed(ctx)? != 0 {
                    ctx.set_i32_at(
                        AppContext::LOSE_NO_PRESS,
                        ctx.i32_at(AppContext::LOSE_NO_PRESS)?.wrapping_add(1),
                    )?;
                    play_sound(sound_manager(ctx)?, 0xb, None);
                }
            }

            let shop_held = touch_is_down(ctx)? != 0 && {
                let x = ctx.i32_at(AppContext::LOSE_SHOP_RECT)?;
                let y = ctx.i32_at(AppContext::LOSE_SHOP_RECT + 4)?;
                let width = ctx.i32_at(AppContext::LOSE_SHOP_RECT + 8)?;
                let height = ctx.i32_at(AppContext::LOSE_SHOP_RECT + 0xc)?;

                hit_test_rect(ctx, x, y, width, height)?
            };

            if shop_held {
                if ctx.u8_at(AppContext::LOSE_SHOP_HELD)? == 0 {
                    play_sound(sound_manager(ctx)?, 0xa, None);
                    ctx.set_block_at::<1>(AppContext::LOSE_SHOP_HELD, [1])?;
                }
            } else {
                ctx.set_block_at::<1>(AppContext::LOSE_SHOP_HELD, [0])?;
            }

            if touch_released(ctx)? == 0 {
                return Ok(true);
            }

            let x = ctx.i32_at(AppContext::LOSE_SHOP_RECT)?;
            let y = ctx.i32_at(AppContext::LOSE_SHOP_RECT + 4)?;
            let width = ctx.i32_at(AppContext::LOSE_SHOP_RECT + 8)?;
            let height = ctx.i32_at(AppContext::LOSE_SHOP_RECT + 0xc)?;

            if !hit_test_rect(ctx, x, y, width, height)? {
                return Ok(true);
            }

            if ctx.i32_at(AppContext::CAT_FOOD_SHOP_ENABLED)? <= 0 {
                return Ok(true);
            }

            play_sound(sound_manager(ctx)?, 0xb, None);
            ctx.set_i32_at(
                AppContext::LOSE_SHOP_PRESS,
                ctx.i32_at(AppContext::LOSE_SHOP_PRESS)?.wrapping_add(1),
            )?;

            Ok(true)
        }
        _ => {
            let shown = ctx.i32_at(AppContext::REWARD_POP_COUNTER)?;
            let mut counter = shown.wrapping_add(1);

            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, counter)?;

            if shown == 0 && ctx.i32_at(AppContext::TUTORIAL_CLEARED)? > 0 {
                play_sound(sound_manager(ctx)?, 0x2a, None);
                counter = ctx.i32_at(AppContext::REWARD_POP_COUNTER)?;
            }

            if (counter as u32) < 5 {
                return Ok(true);
            }

            let x = ctx.i32_at(AppContext::OUTRO_OK_RECT)?;
            let y = ctx.i32_at(AppContext::OUTRO_OK_RECT + 4)?;
            let width = ctx.i32_at(AppContext::OUTRO_OK_RECT + 8)?;
            let height = ctx.i32_at(AppContext::OUTRO_OK_RECT + 0xc)?;

            let hovered = touch_is_down(ctx)? != 0
                && hit_test_rect(ctx, x, y, width, height)?
                && (ctx.i32_at(AppContext::OUTRO_PHASE)? != 4
                    || ctx.i32_at(AppContext::LOSE_TIP_SHOWN)? != 0);

            if hovered {
                if ctx.u8_at(AppContext::DECK_BUTTON_PRESSED)? == 0 {
                    play_sound(sound_manager(ctx)?, 0xa, None);
                    ctx.set_block_at::<1>(AppContext::DECK_BUTTON_PRESSED, [1])?;
                }
            } else {
                ctx.set_block_at::<1>(AppContext::DECK_BUTTON_PRESSED, [0])?;
            }

            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 4)?;

            let map =
                button_bank_find(&ctx.buttons, 0xc8).ok_or(Fault::null_pointer())?;

            new_button_set_touchable(&mut ctx.buttons, map, 0)?;

            let released = touch_released(ctx)? != 0 && {
                let x = ctx.i32_at(AppContext::OUTRO_OK_RECT)?;
                let y = ctx.i32_at(AppContext::OUTRO_OK_RECT + 4)?;
                let width = ctx.i32_at(AppContext::OUTRO_OK_RECT + 8)?;
                let height = ctx.i32_at(AppContext::OUTRO_OK_RECT + 0xc)?;

                hit_test_rect(ctx, x, y, width, height)?
                    || (ctx.i32_at(AppContext::OUTRO_PHASE)? == 4
                        && ctx.i32_at(AppContext::LOSE_TIP_SHOWN)? == 0)
            };

            if (released || back_pressed(ctx)? != 0) && !button_bank_busy(&ctx.buttons)? {
                play_sound(sound_manager(ctx)?, 0xb, None);
                app_on_draw(ctx)?;
                ctx.set_i32_at(AppContext::REVIVE_REQUESTED, 0)?;

                if lose_exit_map_check(ctx)? {
                    battle_check_login_bonus(ctx)?;
                } else {
                    ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1])?;
                    ctx.set_i32_at(AppContext::CURTAIN_STYLE, 1)?;
                }

                return Ok(false);
            }

            if ctx.u8_at(AppContext::CURTAIN_ACTIVE)? != 0 {
                return Ok(true);
            }

            if dialog_top(ctx).is_some() {
                return Ok(true);
            }

            let map =
                button_bank_find(&ctx.buttons, 0xc8).ok_or(Fault::null_pointer())?;

            new_button_set_touchable(&mut ctx.buttons, map, 1)?;

            Ok(true)
        }
    }
}
