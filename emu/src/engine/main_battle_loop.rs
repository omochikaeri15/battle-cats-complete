use std::collections::BTreeMap;

use crate::{Fault, ops};

use super::{
    AppContext, BUTTON_PRESS_BOUNCE, Base, CatStats, Debris, EffectSprite, EnemyStats, Entity,
    VfxSlot, add_attacks_remaining, add_barrier_vfx_frame, add_burrow_count, add_death_timer,
    add_deck_cooldown, add_hp, add_money, add_shield_vfx_frame, add_shockwave_counter,
    add_stage_record, advance_animation_frame, advance_point_decay, app_on_draw,
    attack_dmg_dispatch, back_pressed, background_particles, barrier_vfx_tick,
    base_guard_notice_tick, base_resists_one_shot, base_shake_start, base_shake_tick,
    battle_create_button, battle_init_win, battle_not_finishing, bgm_player_switch,
    bgm_player_tick, call_rng, camera_vertical_correction, cannon_attack, cannon_fire,
    cannon_unlocked, cat_cpu_tick, cat_god_menu_input, cat_hit_executor, cat_update,
    check_collision, clear_barrier_vfx_slot, clear_shield_vfx_slot, clear_zkill_vfx_slot,
    combo_banner_skip_all, combo_banner_update, counter_surge_update, crit_vfx_tick, debris_tick,
    deck_row_swap_tick, demon_battle_banner_tick, deploy_limit_reached, deploy_unit,
    dialog_set_on_draw, dialog_show, dialog_top, does_target, drain_vfx_tick, effect_sprite_init,
    enemy_hit_executor, enemy_schedule_tick, enemy_update, entry_find_by_id, explosion_update,
    fade_update, fever_fade_tick, fever_tick, find_item_index, game_lose_update,
    game_update_lambda_0, game_update_lambda_1, game_update_lambda_2, game_update_lambda_3,
    game_update_lambda_4, game_update_lambda_5, game_update_lambda_6, game_win_update,
    get_anim_len, get_area_attack, get_attack_cooldown, get_attack_count, get_attack_foreswing,
    get_attack_interval, get_attack_only, get_attacks_remaining, get_auto_camera_mode,
    get_barrier_hp, get_barrier_state, get_barrier_vfx_active, get_barrier_vfx_frame,
    get_battle_status, get_boss_type, get_boss_wave_immune, get_bottom_inset_logical,
    get_burrow_count, get_button_unit_form, get_button_unit_id, get_button_unit_row,
    get_cannon_blast_hit, get_castle_enemy_row, get_cat_name, get_crit_vfx, get_death_timer,
    get_deck_cooldown, get_deck_cooldown_max, get_drain_pct, get_drawable_width,
    get_entity_base_idx, get_entity_button, get_entity_frame, get_entity_state, get_frame_damage,
    get_freeze_timer, get_global_map_id, get_hp, get_item_description, get_item_name,
    get_item_selected, get_kb_proc_hit, get_knockbacks, get_map_type, get_max_hp, get_max_zoom,
    get_metal_killer_vfx, get_money, get_money_increment, get_pos_x, get_powerup,
    get_powerup_available, get_prev_curse_timer, get_prev_freeze_timer, get_prev_slow_timer,
    get_prev_weaken_timer, get_savage_blow_vfx, get_score_hit_mask, get_score_time_limit,
    get_setting, get_shield_hp, get_shield_max, get_shield_state, get_shield_vfx,
    get_shield_vfx_frame, get_shockwave_counter, get_slot_unit_id, get_special_rule,
    get_special_rule_params, get_speed, get_stage_index, get_stage_record, get_stage_score,
    get_tech_level, get_text_texture, get_took_damage, get_total_damage_taken, get_toxic_vfx,
    get_trait_dojo, get_unit_recharge, get_warp_timer, get_worker_level, get_worker_upgrade_cost,
    get_zkill_hit, handle_battle_swipe_pinch, has_castle_enemy, has_point_decay, hit_test_rect,
    is_boss, is_score_stage, is_scored_stage, is_touchable_thunk, is_zombie, latch_battle_event,
    load_conjure_desc_textures, log_analytics_event, maanim_get_max_keyframe, max_i32,
    message_layer_clear, message_layer_set, metal_killer_vfx_tick, mission_progress,
    no_more_attacks, on_battle_lost, option_menu_open, option_window_update, orb_deploy_condition,
    pinch_update, play_sound, play_sound_in_battle, point_decay_full, point_lose_update,
    powerup_available, proc_update, query_localizable, record_stage_lineup, record_stage_played,
    recount_deploy_rarities, savage_vfx_tick, scored_map_pays_money, set_attack_cooldown,
    set_auto_camera_mode, set_barrier_hp, set_barrier_state, set_barrier_vfx_active,
    set_barrier_vfx_frame, set_battle_status, set_bgm_duck, set_burrow_start_x, set_crit_vfx,
    set_curse_timer, set_deck_cooldown, set_deck_cooldown_max, set_drain_pct, set_entity_frame,
    set_entity_state, set_frame_damage, set_freeze_timer, set_hit_flash_timer, set_hp, set_max_hp,
    set_metal_killer_vfx, set_no_revive, set_powerup, set_savage_blow_vfx, set_score_hit_mask,
    set_shield_hp, set_shield_state, set_shield_vfx, set_shield_vfx_frame, set_shockwave_counter,
    set_slow_timer, set_stage_record, set_toxic_vfx, set_warp_timer, set_weaken_timer,
    set_zkill_hit, shield_vfx_tick, slot_desc_wait_elapsed, slot_has_flagged_orb, slot_occupied,
    sniper_update, sound_manager, spawn_entity, stage_entry_base_trigger, stage_entry_count,
    stage_entry_is_boss, std_map_int_int_subscript, std_map_int_map_int_subscript,
    std_shared_ptr_texture_assign, std_shared_ptr_texture_reset, std_string_from_cstr,
    surge_update, text_texture_cache, touch_began, touch_is_down, touch_released, toxic_vfx_tick,
    trial_win_update, turn_on_proc_badge, unit_info_select, update_keep_awake, upgrade_worker,
    validate_map_type, vibration_reset, vibration_tick, wave_apply_hits, wave_update,
    worker_unlocked, zkill_vfx_tick,
};

const BGM_SWITCH_RESET: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff];
const SNIPER_BOB_STEP: f32 = 6.0;

pub fn main_battle_loop(ctx: &mut AppContext) -> Result<bool, Fault> {
    let zoom_y = get_setting(&ctx.settings, b"battle_zoom_y", 0x208)?;

    if ctx.u8_at(AppContext::DECK_TWO_LINES)? != 0 {
        let lift = get_setting(&ctx.settings, b"battle_slot_2lines_y", -0x28)?;

        ctx.set_i32_at(AppContext::BATTLE_ZOOM_Y, lift.wrapping_add(zoom_y))?;
    } else {
        ctx.set_i32_at(AppContext::BATTLE_ZOOM_Y, zoom_y)?;
    }

    let inset = get_bottom_inset_logical(ctx)?;

    ctx.deck_bar_base_y = 0x220i32.wrapping_sub(inset);

    if touch_is_down(ctx)? == 0 && touch_released(ctx)? == 0 {
        ctx.set_block_at::<1>(AppContext::DECK_BUTTON_HELD, [0])?;
        ctx.set_i32_at(AppContext::DECK_HOLD_FRAMES, 0)?;
    }

    if ctx.u8_at(AppContext::DECK_BUTTON_HELD)? == 0 {
        let release = ctx.i32_at(AppContext::DECK_HOLD_RELEASE)?;

        if release > 0 {
            ctx.set_i32_at(AppContext::DECK_HOLD_RELEASE, release.wrapping_sub(1))?;

            if release.wrapping_sub(1) == 0 {
                ctx.set_i32_at(AppContext::DECK_HOLD_SLOT, -1)?;
            }
        }
    }

    base_shake_tick(ctx)?;

    if get_battle_status(ctx)? == 0 {
        vibration_tick(ctx)?;
    }

    'frame: {
        if ctx.u8_at(AppContext::UNIT_INFO_OVERLAY_OPEN)? != 0 {
            let mut slot = 0usize;

            loop {
                let unit_id = get_button_unit_id(ctx, 0, slot as i32)?;

                if unit_id < 0 {
                    break 'frame;
                }

                let mut shift = 0i32;

                if slot <= 4 {
                    shift = get_setting(&ctx.settings, b"battle_slot_2lines_line", 0x5a)?
                        .wrapping_neg();
                }

                'button: {
                    if touch_is_down(ctx)? == 0 {
                        break 'button;
                    }

                    let column = *ctx.deck_button_x.get(slot).ok_or(Fault::index_out_of_range(slot as i64, 10))? as f64;
                    let x = ops::cvttsd2si(
                        get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + column,
                    );
                    let y = ctx
                        .i32_at(AppContext::LETTERBOX_SHIFT)?
                        .wrapping_add(ctx.deck_bar_base_y.wrapping_add(shift))
                        .wrapping_sub(1);

                    if !hit_test_rect(ctx, x, y, 0x6e, 0x58)? {
                        break 'button;
                    }

                    if ctx.i32_at(AppContext::UNIT_INFO_SLOT)? as u32 as usize == slot {
                        break 'button;
                    }

                    ctx.set_i32_at(AppContext::UNIT_INFO_SLOT, slot as i32)?;
                    unit_info_select(ctx, slot as i32)?;
                    play_sound(sound_manager(ctx)?, 0xa, None);

                    break 'frame;
                }

                slot += 1;

                if slot >= (ctx.u8_at(AppContext::DECK_TWO_LINES)? as usize) * 5 + 5 {
                    break 'frame;
                }
            }
        }

        'menus: {
            if ctx.u8_at(AppContext::OPTION_MENU_IS_OPEN)? != 0 {
                if ctx.u8_at(AppContext::CURTAIN_ACTIVE)? == 0 {
                    option_window_update();

                    break 'frame;
                }

                break 'menus;
            }

            if ctx.u8_at(AppContext::TUTORIAL_POPUP_OPEN)? != 0 {
                break 'menus;
            }

            if ctx.u8_at(AppContext::CAT_GOD_MENU_IS_OPEN)? == 0 {
                let intro = ctx.i32_at(AppContext::BATTLE_INTRO_FRAME)?;

                ctx.set_i32_at(
                    AppContext::BATTLE_INTRO_FRAME,
                    if intro < 0x149 {
                        intro.wrapping_add(1)
                    } else {
                        0x14a
                    },
                )?;

                let offset_x = (get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5) as f32;

                ctx.set_f32_at(AppContext::CAMERA_OFFSET, offset_x)?;
                ctx.set_f32_at(
                    AppContext::CAMERA_OFFSET.wrapping_add(4),
                    ctx.i32_at(AppContext::BATTLE_ZOOM_Y)? as f32,
                )?;

                let scale = ctx.i32_at(AppContext::CAMERA_ZOOM)? as f32 / 10000.0;
                let shear = 0.0f32 * scale;

                ctx.set_f32_at(AppContext::CAMERA_MATRIX, scale)?;
                ctx.set_f32_at(AppContext::CAMERA_MATRIX.wrapping_add(4), shear)?;
                ctx.set_f32_at(AppContext::CAMERA_MATRIX.wrapping_add(8), shear)?;
                ctx.set_f32_at(AppContext::CAMERA_MATRIX.wrapping_add(0xc), scale)?;

                let pan_x = (0x3c0i32.wrapping_sub(get_drawable_width(ctx)?) as f64 * 0.5) as f32;
                let base_y = ctx.i32_at(AppContext::BATTLE_ZOOM_Y)?;
                let pan_y = camera_vertical_correction(ctx)?.wrapping_sub(base_y) as f32;
                let m0 = ctx.f32_at(AppContext::CAMERA_MATRIX)?;
                let m1 = ctx.f32_at(AppContext::CAMERA_MATRIX.wrapping_add(4))?;
                let m2 = ctx.f32_at(AppContext::CAMERA_MATRIX.wrapping_add(8))?;
                let m3 = ctx.f32_at(AppContext::CAMERA_MATRIX.wrapping_add(0xc))?;
                let e0 = ctx.f32_at(AppContext::CAMERA_OFFSET)?;
                let e1 = ctx.f32_at(AppContext::CAMERA_OFFSET.wrapping_add(4))?;

                ctx.set_f32_at(AppContext::CAMERA_OFFSET, pan_x * m0 + m2 * pan_y + e0)?;
                ctx.set_f32_at(
                    AppContext::CAMERA_OFFSET.wrapping_add(4),
                    pan_x * m1 + m3 * pan_y + e1,
                )?;

                pinch_update(ctx, AppContext::PINCH)?;
                ctx.zero(AppContext::PROC_ROLLS, 0x30)?;

                let zoom_percent = ops::div_100(ctx.i32_at(AppContext::CAMERA_ZOOM)?);

                ctx.set_i32_at(AppContext::DRAW_TEMP_1, zoom_percent)?;
                ctx.set_block_at::<1>(AppContext::BASE_KILL_BLOCKED, [0])?;

                'camera: {
                    if get_auto_camera_mode(ctx)? == 0 {
                        break 'camera;
                    }

                    if get_auto_camera_mode(ctx)? == 1 {
                        let scale = zoom_percent as f32 / 100.0;
                        let stage = ctx.i32_at(AppContext::STAGE_LENGTH)? as f32;
                        let distance = ops::cvttss2si((stage * scale + -9600.0) / scale)
                            .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?);

                        if get_battle_status(ctx)? == 0 {
                            if distance <= 0x51 {
                                let x = ctx.i32_at(AppContext::STAGE_LENGTH)?.wrapping_add(-0x2580);

                                ctx.set_i32_at(AppContext::CAMERA_X, x)?;
                                set_auto_camera_mode(ctx, 0)?;
                            } else {
                                let x = ctx
                                    .i32_at(AppContext::CAMERA_X)?
                                    .wrapping_add(((distance as u32) >> 1) as i32);

                                ctx.set_i32_at(AppContext::CAMERA_X, x)?;
                            }

                            break 'camera;
                        }

                        if distance <= 0x51 {
                            let zoom = ctx.i32_at(AppContext::CAMERA_ZOOM)?;

                            if zoom >= get_max_zoom(ctx)? {
                                let max = get_max_zoom(ctx)?;

                                ctx.set_i32_at(AppContext::CAMERA_ZOOM, max)?;

                                let x = ctx.i32_at(AppContext::STAGE_LENGTH)?.wrapping_add(-0x2580);

                                ctx.set_i32_at(AppContext::CAMERA_X, x)?;
                                set_auto_camera_mode(ctx, 0)?;

                                break 'camera;
                            }
                        }

                        let x = ops::div_2(distance)
                            .wrapping_add(ctx.i32_at(AppContext::CAMERA_X)?);

                        ctx.set_i32_at(AppContext::CAMERA_X, x)?;

                        let stage_length = ctx.i32_at(AppContext::STAGE_LENGTH)?;
                        let limit =
                            ops::cvttss2si((stage_length as f32 * scale + -9600.0) / scale);

                        if x >= limit {
                            ctx.set_block_at::<1>(AppContext::AUTO_CAMERA_ARRIVED, [1])?;
                        } else if ctx.u8_at(AppContext::AUTO_CAMERA_ARRIVED)? == 0 {
                            break 'camera;
                        }

                        ctx.set_i32_at(AppContext::CAMERA_X, stage_length.wrapping_add(-0x2580))?;

                        break 'camera;
                    }

                    if get_auto_camera_mode(ctx)? != 2 {
                        break 'camera;
                    }

                    let x = ctx.i32_at(AppContext::CAMERA_X)?;

                    if get_battle_status(ctx)? == 0 {
                        if x > 0x51 {
                            ctx.set_i32_at(
                                AppContext::CAMERA_X,
                                ctx.i32_at(AppContext::CAMERA_X)?
                                    .wrapping_sub(((x as u32) >> 1) as i32),
                            )?;

                            break 'camera;
                        }

                        ctx.set_i32_at(AppContext::CAMERA_X, 0)?;
                        set_auto_camera_mode(ctx, 0)?;

                        break 'camera;
                    }

                    if x <= 0x51 {
                        let zoom = ctx.i32_at(AppContext::CAMERA_ZOOM)?;

                        if zoom >= get_max_zoom(ctx)? {
                            ctx.set_i32_at(AppContext::CAMERA_X, 0)?;
                            set_auto_camera_mode(ctx, 0)?;

                            break 'camera;
                        }
                    }

                    ctx.set_i32_at(
                        AppContext::CAMERA_X,
                        ctx.i32_at(AppContext::CAMERA_X)?
                            .wrapping_sub(ops::div_2(x)),
                    )?;
                }

                let mut skip_banner = false;

                if get_battle_status(ctx)? == 1 {
                    if ctx.u8_at(AppContext::CURTAIN_ACTIVE)? == 0 && !game_win_update(ctx)? {
                        return Ok(false);
                    }

                    skip_banner = true;
                } else if get_battle_status(ctx)? == 2 {
                    if ctx.u8_at(AppContext::CURTAIN_ACTIVE)? == 0 && !game_lose_update(ctx)? {
                        return Ok(false);
                    }

                    skip_banner = true;
                } else if get_battle_status(ctx)? == 3 {
                    ctx.set_i32_at(
                        AppContext::SETUP_FRAMES,
                        ctx.i32_at(AppContext::SETUP_FRAMES)?.wrapping_add(1),
                    )?;

                    if ctx.u8_at(AppContext::CURTAIN_ACTIVE)? == 0 {
                        ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
                        set_battle_status(ctx, 0)?;
                        ctx.set_i32_at(AppContext::SETUP_FRAMES, 0)?;
                    }
                } else if get_battle_status(ctx)? == 4 {
                    if ctx.u8_at(AppContext::CURTAIN_ACTIVE)? == 0 && !trial_win_update(ctx)? {
                        return Ok(false);
                    }

                    skip_banner = true;
                } else if get_battle_status(ctx)? == 7 {
                    if ctx.u8_at(AppContext::CURTAIN_ACTIVE)? == 0 && !point_lose_update(ctx)? {
                        return Ok(false);
                    }

                    skip_banner = true;
                } else {
                    'input: {
                        'tutorial: {
                            if ctx.i32_at(AppContext::TUTORIAL_CLEARED)? != 0 {
                                if get_global_map_id(ctx, 0)? == 0xbb8
                                    && get_stage_index(ctx)? == 1
                                    && get_stage_record(ctx, -2, 0, 1, 0, 0)? == 0
                                    && ctx.i32_at(AppContext::BATTLE_FRAME_COUNTER)? >= 0x18
                                    && ctx.i32_at(AppContext::TUTORIAL_STEP)? == 0
                                {
                                    let key = std_string_from_cstr(b"battle_tutorial_2");
                                    let text = query_localizable(ctx, &key);
                                    let dialog = dialog_show(
                                        ctx,
                                        &text,
                                        0,
                                        0,
                                        4,
                                        Some(game_update_lambda_3),
                                    )?;

                                    dialog_set_on_draw(ctx, dialog, Some(game_update_lambda_4));
                                    ctx.set_i32_at(
                                        AppContext::TUTORIAL_STEP,
                                        ctx.i32_at(AppContext::TUTORIAL_STEP)?.wrapping_add(1),
                                    )?;
                                    ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [1])?;
                                    app_on_draw(ctx)?;

                                    return Ok(false);
                                }

                                if get_global_map_id(ctx, 0)? == 0xbb8
                                    && get_stage_index(ctx)? == 2
                                    && get_stage_record(ctx, -2, 0, 2, 0, 0)? == 0
                                {
                                    let frames = ctx.i32_at(AppContext::BATTLE_FRAME_COUNTER)?;
                                    let key = std_string_from_cstr(b"tutorial_duration1");
                                    let due = frames >= get_setting(&ctx.settings, &key, 0x1e)?
                                        && ctx.i32_at(AppContext::TUTORIAL_STEP)? == 0;

                                    if due {
                                        let key = std_string_from_cstr(b"battle_tutorial_3");
                                        let text = query_localizable(ctx, &key);
                                        let dialog = dialog_show(
                                            ctx,
                                            &text,
                                            0,
                                            0,
                                            4,
                                            Some(game_update_lambda_5),
                                        )?;

                                        dialog_set_on_draw(ctx, dialog, Some(game_update_lambda_6));
                                        ctx.set_i32_at(
                                            AppContext::TUTORIAL_STEP,
                                            ctx.i32_at(AppContext::TUTORIAL_STEP)?.wrapping_add(1),
                                        )?;
                                        ctx.set_block_at::<1>(
                                            AppContext::TUTORIAL_POPUP_OPEN,
                                            [1],
                                        )?;
                                        app_on_draw(ctx)?;

                                        return Ok(false);
                                    }
                                }

                                for (seen, two_rows) in [
                                    (AppContext::TUTORIAL_DECK_SEEN, false),
                                    (AppContext::TUTORIAL_TWO_ROWS_SEEN, true),
                                ] {
                                    if ctx.i32_at(seen)? != 0 {
                                        continue;
                                    }

                                    let mut filled = 0u32;

                                    for button in 0..10 {
                                        filled = filled.wrapping_add(
                                            (get_button_unit_row(ctx, 0, button)? != -1) as u32,
                                        );

                                        if filled < 6 {
                                            continue;
                                        }

                                        let touched = touch_is_down(ctx)? != 0
                                            || touch_began(ctx)? != 0
                                            || touch_released(ctx)? != 0;

                                        if !touched {
                                            let timer = ctx.i32_at(AppContext::TUTORIAL_TIMER)?;

                                            ctx.set_i32_at(
                                                AppContext::TUTORIAL_TIMER,
                                                timer.wrapping_add(1),
                                            )?;

                                            if timer < 0x17 {
                                                continue;
                                            }
                                        }

                                        app_on_draw(ctx)?;
                                        ctx.set_i32_at(seen, 1)?;
                                        ctx.set_block_at::<1>(
                                            AppContext::TUTORIAL_POPUP_OPEN,
                                            [1],
                                        )?;
                                        ctx.set_block_at::<16>(
                                            AppContext::TUTORIAL_TIMER,
                                            [0; 16],
                                        )?;

                                        if ctx.i32_at(AppContext::BGM_SWITCH_STATE)? == 0 {
                                            set_bgm_duck(sound_manager(ctx)?, 0x32);
                                        } else if !two_rows {
                                            return Ok(false);
                                        }

                                        if two_rows {
                                            ctx.set_block_at::<1>(AppContext::DECK_TWO_LINES, [1])?;
                                        }

                                        return Ok(false);
                                    }
                                }

                                break 'tutorial;
                            }

                            let frames = ctx.i32_at(AppContext::BATTLE_FRAME_COUNTER)?;
                            let key = std_string_from_cstr(b"tutorial_duration2");

                            if frames >= get_setting(&ctx.settings, &key, 0x96)?
                                && ctx.i32_at(AppContext::TUTORIAL_STEP)? == 0
                            {
                                let key = std_string_from_cstr(b"battle_tutorial_0");
                                let text = query_localizable(ctx, &key);

                                dialog_show(ctx, &text, 0, 0, 0, Some(game_update_lambda_0))?;
                                ctx.set_i32_at(
                                    AppContext::TUTORIAL_STEP,
                                    ctx.i32_at(AppContext::TUTORIAL_STEP)?.wrapping_add(1),
                                )?;
                                ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [1])?;
                                app_on_draw(ctx)?;

                                return Ok(false);
                            }

                            let frames = ctx.i32_at(AppContext::BATTLE_FRAME_COUNTER)?;
                            let key = std_string_from_cstr(b"tutorial_duration3");

                            if frames >= get_setting(&ctx.settings, &key, 0x1c2)?
                                && dialog_top(ctx).is_none()
                                && ctx.i32_at(AppContext::TUTORIAL_STEP)? == 1
                            {
                                let key = std_string_from_cstr(b"battle_tutorial_1");
                                let text = query_localizable(ctx, &key);
                                let dialog = dialog_show(
                                    ctx,
                                    &text,
                                    0,
                                    0,
                                    0x24,
                                    Some(game_update_lambda_1),
                                )?;

                                dialog_set_on_draw(ctx, dialog, Some(game_update_lambda_2));
                                ctx.set_i32_at(
                                    AppContext::TUTORIAL_STEP,
                                    ctx.i32_at(AppContext::TUTORIAL_STEP)?.wrapping_add(1),
                                )?;
                                ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [1])?;
                                app_on_draw(ctx)?;

                                return Ok(false);
                            }
                        }

                        let special_mode = ctx.i32_at(AppContext::ENTRY_STAGE)? == 0x30
                            || ctx.i32_at(AppContext::CHAPTER_MODE)? == 3
                            || ctx.i32_at(AppContext::CHAPTER_MODE)? == 0x63;

                        let cat_god_intro = !special_mode
                            && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? == 0
                            && ctx.u8_at(AppContext::BATTLE_IS_INVASION)? == 0
                            && ctx.u8_at(AppContext::BATTLE_IS_Z_INVASION)? == 0
                            && !(get_map_type(ctx, 0)? == -7 && get_stage_index(ctx)? == 0x2f)
                            && ctx.i32_at(AppContext::TUTORIAL_CAT_GOD_SEEN)? == 0
                            && ctx.i32_at(AppContext::CAT_GOD_AVAILABLE)? > 0;

                        if cat_god_intro {
                            let touched = touch_is_down(ctx)? != 0
                                || touch_began(ctx)? != 0
                                || touch_released(ctx)? != 0;
                            let mut due = touched;

                            if !touched {
                                let timer = ctx.i32_at(AppContext::TUTORIAL_TIMER)?;

                                ctx.set_i32_at(AppContext::TUTORIAL_TIMER, timer.wrapping_add(1))?;
                                due = timer >= 0x17;
                            }

                            if due {
                                app_on_draw(ctx)?;
                                ctx.set_i32_at(AppContext::TUTORIAL_CAT_GOD_SEEN, 1)?;
                                ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [1])?;
                                ctx.set_block_at::<16>(AppContext::TUTORIAL_TIMER, [0; 16])?;

                                if ctx.i32_at(AppContext::BGM_SWITCH_STATE)? == 0 {
                                    set_bgm_duck(sound_manager(ctx)?, 0x32);
                                }

                                return Ok(false);
                            }
                        }

                        let press = ctx.i32_at(AppContext::CAT_GOD_BUTTON_PRESS)?;

                        if press > 0 {
                            ctx.set_i32_at(
                                AppContext::CAT_GOD_BUTTON_PRESS,
                                press.wrapping_add(1),
                            )?;

                            if (press as u32) < 5 {
                                break 'input;
                            }

                            ctx.set_i32_at(AppContext::CAT_GOD_BUTTON_PRESS, 0)?;

                            if ctx.i32_at(AppContext::ENTRY_STAGE)? == 0x30
                                || ctx.i32_at(AppContext::CHAPTER_MODE)? == 3
                                || ctx.i32_at(AppContext::CHAPTER_MODE)? == 0x63
                            {
                                break 'input;
                            }

                            app_on_draw(ctx)?;
                            ctx.set_block_at::<1>(AppContext::CAT_GOD_MENU_IS_OPEN, [1])?;
                            bgm_player_switch(ctx, 1, 1)?;

                            if (ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)?.wrapping_sub(1) as u32)
                                <= 1
                            {
                                ctx.set_i32_at(AppContext::CAT_GOD_INTRO_STEP, 0)?;
                            }

                            return Ok(false);
                        }

                        let press = ctx.i32_at(AppContext::PAUSE_PRESS)?;

                        if press > 0 {
                            ctx.set_i32_at(AppContext::PAUSE_PRESS, press.wrapping_add(1))?;

                            if (press as u32) < 5 {
                                break 'input;
                            }

                            ctx.set_i32_at(AppContext::PAUSE_PRESS, 0)?;
                            app_on_draw(ctx)?;
                            ctx.set_block_at::<1>(AppContext::OPTION_MENU_IS_OPEN, [1])?;
                            update_keep_awake(ctx, 1)?;
                            option_menu_open(ctx)?;

                            if ctx.i32_at(AppContext::BGM_SWITCH_STATE)? == 0 {
                                set_bgm_duck(sound_manager(ctx)?, 0x32);
                            }

                            return Ok(false);
                        }

                        let press = ctx.i32_at(AppContext::SPEED_UP_PRESS)?;

                        if press > 0 {
                            ctx.set_i32_at(AppContext::SPEED_UP_PRESS, press.wrapping_add(1))?;

                            if (press as u32) < 5 {
                                break 'input;
                            }

                            ctx.set_i32_at(AppContext::SPEED_UP_PRESS, 0)?;

                            if get_powerup_available(ctx)? != 0
                                && get_powerup(ctx, 0)?
                                && ctx.u8_at(AppContext::SPEED_UP_LATCH)? == 0
                            {
                                ctx.set_block_at::<1>(AppContext::SPEED_UP_LATCH, [1])?;
                            } else {
                                let on = get_powerup(ctx, 0)?;

                                set_powerup(ctx, 0, (!on) as i32)?;
                                ctx.set_block_at::<1>(AppContext::SPEED_UP_LATCH, [0])?;
                            }

                            if ctx.i32_at(AppContext::TUTORIAL_CLEARED)? == 0
                                && ctx.i32_at(AppContext::TUTORIAL_STEP)? == 3
                            {
                                ctx.set_i32_at(AppContext::TUTORIAL_STEP, 4)?;
                            }

                            break 'input;
                        }

                        let press = ctx.i32_at(AppContext::CPU_PRESS)?;

                        if press > 0 {
                            ctx.set_i32_at(AppContext::CPU_PRESS, press.wrapping_add(1))?;

                            if (press as u32) < 5 {
                                break 'input;
                            }

                            ctx.set_i32_at(AppContext::CPU_PRESS, 0)?;

                            let on = get_powerup(ctx, 3)?;

                            set_powerup(ctx, 3, (!on) as i32)?;

                            break 'input;
                        }

                        let press = ctx.i32_at(AppContext::SNIPER_PRESS)?;

                        if press > 0 {
                            ctx.set_i32_at(AppContext::SNIPER_PRESS, press.wrapping_add(1))?;

                            if (press as u32) < 5 {
                                break 'input;
                            }

                            ctx.set_i32_at(AppContext::SNIPER_PRESS, 0)?;

                            let on = get_powerup(ctx, 5)?;

                            set_powerup(ctx, 5, (!on) as i32)?;

                            break 'input;
                        }

                        let mut pressing = false;

                        for button in 0..10usize {
                            if ctx.i32_at(AppContext::DECK_PRESS.wrapping_add(button * 4))? > 0 {
                                pressing = true;
                            }
                        }

                        if pressing {
                            let buttons = (ctx.u8_at(AppContext::DECK_TWO_LINES)? as usize) * 5 + 5;
                            let mut button = 0usize;

                            while button < buttons
                                && ctx.i32_at(AppContext::DECK_PRESS.wrapping_add(button * 4))? <= 0
                            {
                                button += 1;
                            }

                            let counter = AppContext::DECK_PRESS.wrapping_add(button * 4);
                            let frames = ctx.i32_at(counter)?.wrapping_add(1);

                            ctx.set_i32_at(counter, frames)?;

                            if (frames as u32) < 6 {
                                break 'input;
                            }

                            ctx.set_i32_at(counter, 0)?;
                            recount_deploy_rarities(ctx)?;

                            let row = ctx.i32_at(AppContext::DECK_ROW_SHOWN)?;

                            if row == 0 || ctx.u8_at(AppContext::DECK_TWO_LINES)? != 0 {
                                deploy_unit(ctx, 0, button as i32, 1)?;
                            } else if row == 1 {
                                deploy_unit(ctx, 0, (button as i32).wrapping_add(5), 1)?;
                            }

                            break 'input;
                        }

                        if ctx.u8_at(AppContext::INPUT_BLOCKED)? == 0 {
                            handle_battle_swipe_pinch(ctx)?;
                        }

                        cat_cpu_tick(ctx, 0)?;

                        let half = ops::div_2(ctx.i32_at(AppContext::STAGE_LENGTH)?)
                            .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?);
                        let column = ops::div_10(half).wrapping_add(-0x41) as f64;
                        let x = ((get_drawable_width(ctx)?.wrapping_add(-0x3c0)) as f64 * 0.5
                            + column) as f32;
                        let press = ctx.i32_at(AppContext::CAT_GOD_BUTTON_PRESS)?;
                        let bounce = ops::div_2(
                            *BUTTON_PRESS_BOUNCE.get(press as i64 as usize).ok_or(
                                Fault::index_out_of_range(press as i64, 10),
                            )?,
                        );
                        let lift = ctx
                            .i32_at(AppContext::LETTERBOX_SHIFT)?
                            .wrapping_add(ctx.i32_at(AppContext::CAT_GOD_BUTTON_SINK)?)
                            .wrapping_add(ctx.i32_at(AppContext::DECK_BAR_SLIDE)?)
                            .wrapping_add(bounce);
                        let y = (-0x18i32).wrapping_sub(lift) as f32;
                        let far_x = x + 131.0;
                        let far_y = 131.0 + y;
                        let m0 = ctx.f32_at(AppContext::CAMERA_MATRIX)?;
                        let m1 = ctx.f32_at(AppContext::CAMERA_MATRIX.wrapping_add(4))?;
                        let m2 = ctx.f32_at(AppContext::CAMERA_MATRIX.wrapping_add(8))?;
                        let m3 = ctx.f32_at(AppContext::CAMERA_MATRIX.wrapping_add(0xc))?;
                        let e0 = ctx.f32_at(AppContext::CAMERA_OFFSET)?;
                        let e1 = ctx.f32_at(AppContext::CAMERA_OFFSET.wrapping_add(4))?;
                        let near = [x * m0 + y * m2 + e0, x * m1 + y * m3 + e1];
                        let far = [far_x * m0 + far_y * m2 + e0, far_x * m1 + far_y * m3 + e1];

                        ctx.set_i32_at(
                            AppContext::CAT_GOD_BUTTON_RECT,
                            ops::cvttss2si(near[0]),
                        )?;
                        ctx.set_i32_at(
                            AppContext::CAT_GOD_BUTTON_RECT.wrapping_add(4),
                            ops::cvttss2si(near[1]),
                        )?;
                        ctx.set_i32_at(
                            AppContext::CAT_GOD_BUTTON_RECT.wrapping_add(8),
                            ops::cvttss2si(far[0] - near[0]),
                        )?;
                        ctx.set_i32_at(
                            AppContext::CAT_GOD_BUTTON_RECT.wrapping_add(0xc),
                            ops::cvttss2si(far[1] - near[1]),
                        )?;

                        if touch_is_down(ctx)? != 0 {
                            if ctx.u8_at(AppContext::DECK_ROW_SWAPPING)? != 0
                                || ctx.u8_at(AppContext::CAMERA_DRAGGING)? != 0
                            {
                                ctx.set_block_at::<1>(AppContext::TOUCH_CAPTURED, [1])?;
                            }
                        } else if touch_released(ctx)? == 0 {
                            ctx.set_block_at::<1>(AppContext::TOUCH_CAPTURED, [0])?;
                        }

                        if ctx.i32_at(AppContext::SWIPE_VELOCITY)? == 0
                            && ctx.u8_at(AppContext::PINCH_ZOOMED)? == 0
                        {
                            for item in 0..6i32 {
                                if touch_is_down(ctx)? == 0 {
                                    continue;
                                }

                                let rect =
                                    AppContext::ITEM_RECTS.wrapping_add(item as usize * 0x10);
                                let (rx, ry, rw, rh) = (
                                    ctx.i32_at(rect)?,
                                    ctx.i32_at(rect.wrapping_add(4))?,
                                    ctx.i32_at(rect.wrapping_add(8))?,
                                    ctx.i32_at(rect.wrapping_add(0xc))?,
                                );

                                if !hit_test_rect(ctx, rx, ry, rw, rh)? {
                                    continue;
                                }

                                if !powerup_available(ctx, item)? {
                                    continue;
                                }

                                if !(ctx.i32_at(AppContext::TOOLTIP_ITEM)? as u32 as i32 == item
                                    && ctx.u8_at(AppContext::INPUT_BLOCKED)? != 0)
                                {
                                    ctx.set_i32_at(AppContext::TOOLTIP_ITEM, item)?;

                                    let index = find_item_index(
                                        ctx,
                                        ctx.i32_at(AppContext::TOOLTIP_ITEM)?,
                                    )?;
                                    let name = get_item_name(ctx, index);
                                    let font = ctx.default_font.clone();
                                    let texture = get_text_texture(
                                        text_texture_cache(ctx)?,
                                        &name,
                                        &font,
                                        0x1e,
                                        1,
                                        0,
                                    );
                                    let page = ctx.i32_at(AppContext::TOOLTIP_PAGE)? as i64;
                                    let limit = ctx.label_texts.len() as i64;
                                    let slot = ctx.label_texts.get_mut(page as usize).ok_or(
                                        Fault::index_out_of_range(page, limit),
                                    )?;

                                    std_shared_ptr_texture_assign(slot, Some(texture));

                                    let mut line = page.wrapping_sub(1);
                                    let mut offset = 0usize;

                                    loop {
                                        let index = find_item_index(
                                            ctx,
                                            ctx.i32_at(AppContext::TOOLTIP_ITEM)?,
                                        )?;
                                        let lines = get_item_description(ctx, index);
                                        let text = lines.get(offset).cloned().unwrap_or_default();
                                        let texture = get_text_texture(
                                            text_texture_cache(ctx)?,
                                            &text,
                                            &font,
                                            0x1e,
                                            1,
                                            0,
                                        );
                                        let target = (line.wrapping_add(2)) as usize;
                                        let limit = ctx.label_texts.len() as i64;
                                        let slot = ctx.label_texts.get_mut(target).ok_or(
                                            Fault::index_out_of_range(target as i64, limit),
                                        )?;

                                        std_shared_ptr_texture_assign(slot, Some(texture));
                                        line += 1;
                                        offset += 1;

                                        if line >= ctx.i32_at(AppContext::TOOLTIP_PAGE)? as i64 + 2
                                        {
                                            break;
                                        }
                                    }

                                    if ctx.i32_at(AppContext::TOOLTIP_ITEM)? == 0
                                        && ctx.u8_at(AppContext::SPEED_UP_LATCH)? == 0
                                    {
                                        message_layer_clear(ctx, 0)?;

                                        let key = std_string_from_cstr(b"speedup_message");
                                        let text = query_localizable(ctx, &key);

                                        message_layer_set(ctx, 0, &text, 0x1e, 0x2c6)?;
                                    }
                                }

                                ctx.set_block_at::<1>(AppContext::INPUT_BLOCKED, [1])?;
                            }

                            if ctx.u8_at(AppContext::INPUT_BLOCKED)? != 0 {
                                ctx.set_i32_at(AppContext::UI_TAP_LOCKOUT, 5)?;
                            }
                        }

                        if ctx.i32_at(AppContext::SWIPE_VELOCITY)? != 0
                            || ctx.u8_at(AppContext::PINCH_ZOOMED)? != 0
                        {
                            break 'input;
                        }

                        if ctx.u8_at(AppContext::CPU_ENABLED)? == 0 {
                            let (rx, ry, rw, rh) = (
                                ctx.i32_at(AppContext::CANNON_RECT)?,
                                ctx.i32_at(AppContext::CANNON_RECT.wrapping_add(4))?,
                                ctx.i32_at(AppContext::CANNON_RECT.wrapping_add(8))?,
                                ctx.i32_at(AppContext::CANNON_RECT.wrapping_add(0xc))?,
                            );

                            if cannon_unlocked(ctx)?
                                && touch_is_down(ctx)? != 0
                                && hit_test_rect(ctx, rx, ry, rw, rh)?
                            {
                                if ctx.u8_at(AppContext::CANNON_HELD)? == 0 {
                                    play_sound(sound_manager(ctx)?, 0xa, None);
                                    ctx.set_block_at::<1>(AppContext::CANNON_HELD, [1])?;
                                }
                            } else {
                                ctx.set_block_at::<1>(AppContext::CANNON_HELD, [0])?;
                            }

                            let (rx, ry, rw, rh) = (
                                ctx.i32_at(AppContext::WORKER_RECT)?,
                                ctx.i32_at(AppContext::WORKER_RECT.wrapping_add(4))?,
                                ctx.i32_at(AppContext::WORKER_RECT.wrapping_add(8))?,
                                ctx.i32_at(AppContext::WORKER_RECT.wrapping_add(0xc))?,
                            );

                            if worker_unlocked(ctx)?
                                && touch_is_down(ctx)? != 0
                                && hit_test_rect(ctx, rx, ry, rw, rh)?
                            {
                                if ctx.u8_at(AppContext::WORKER_HELD)? == 0 {
                                    play_sound(sound_manager(ctx)?, 0xa, None);
                                    ctx.set_block_at::<1>(AppContext::WORKER_HELD, [1])?;
                                }
                            } else {
                                ctx.set_block_at::<1>(AppContext::WORKER_HELD, [0])?;
                            }
                        }

                        let (px, py, pw, ph) = (
                            ctx.i32_at(AppContext::PAUSE_RECT)?,
                            ctx.i32_at(AppContext::PAUSE_RECT.wrapping_add(4))?,
                            ctx.i32_at(AppContext::PAUSE_RECT.wrapping_add(8))?,
                            ctx.i32_at(AppContext::PAUSE_RECT.wrapping_add(0xc))?,
                        );

                        if touch_is_down(ctx)? != 0 && hit_test_rect(ctx, px, py, pw, ph)? {
                            if ctx.u8_at(AppContext::PAUSE_HELD)? == 0 {
                                play_sound(sound_manager(ctx)?, 0xa, None);
                                ctx.set_block_at::<1>(AppContext::PAUSE_HELD, [1])?;
                            }
                        } else {
                            ctx.set_block_at::<1>(AppContext::PAUSE_HELD, [0])?;
                        }

                        let (rx, ry, rw, rh) = (
                            ctx.i32_at(AppContext::CANNON_RECT)?,
                            ctx.i32_at(AppContext::CANNON_RECT.wrapping_add(4))?,
                            ctx.i32_at(AppContext::CANNON_RECT.wrapping_add(8))?,
                            ctx.i32_at(AppContext::CANNON_RECT.wrapping_add(0xc))?,
                        );

                        if cannon_unlocked(ctx)?
                            && touch_released(ctx)? != 0
                            && hit_test_rect(ctx, rx, ry, rw, rh)?
                            && ctx.u8_at(AppContext::INPUT_BLOCKED)? == 0
                        {
                            cannon_fire(ctx, 1)?;

                            break 'input;
                        }

                        let (rx, ry, rw, rh) = (
                            ctx.i32_at(AppContext::WORKER_RECT)?,
                            ctx.i32_at(AppContext::WORKER_RECT.wrapping_add(4))?,
                            ctx.i32_at(AppContext::WORKER_RECT.wrapping_add(8))?,
                            ctx.i32_at(AppContext::WORKER_RECT.wrapping_add(0xc))?,
                        );

                        if worker_unlocked(ctx)?
                            && touch_released(ctx)? != 0
                            && hit_test_rect(ctx, rx, ry, rw, rh)?
                            && ctx.u8_at(AppContext::INPUT_BLOCKED)? == 0
                        {
                            ctx.set_i32_at(AppContext::UI_TAP_LOCKOUT, 0)?;
                            ctx.set_block_at::<1>(AppContext::INPUT_BLOCKED, [0])?;

                            let wallet = AppContext::faction_flags(0);

                            if get_worker_level(ctx, wallet)? != 7
                                && get_money(ctx, wallet)? >= get_worker_upgrade_cost(ctx, wallet)?
                            {
                                ctx.set_i32_at(AppContext::CPU_PENDING_ACTION, 0)?;

                                let cost = get_worker_upgrade_cost(ctx, wallet)?;

                                add_money(ctx, wallet, cost.wrapping_neg())?;
                                upgrade_worker(ctx, wallet)?;
                                ctx.set_i32_at(AppContext::WORKER_UPGRADE_VFX, 0xe)?;
                                play_sound(sound_manager(ctx)?, 0x13, None);
                            } else {
                                play_sound(sound_manager(ctx)?, 0xf, None);
                            }

                            break 'input;
                        }

                        let pause_tapped = (touch_released(ctx)? != 0
                            && hit_test_rect(ctx, px, py, pw, ph)?)
                            || back_pressed(ctx)? != 0;

                        if pause_tapped && ctx.u8_at(AppContext::INPUT_BLOCKED)? == 0 {
                            play_sound(sound_manager(ctx)?, 0xb, None);
                            ctx.set_i32_at(AppContext::UI_TAP_LOCKOUT, 0)?;
                            ctx.set_block_at::<1>(AppContext::INPUT_BLOCKED, [0])?;
                            ctx.set_i32_at(
                                AppContext::PAUSE_PRESS,
                                ctx.i32_at(AppContext::PAUSE_PRESS)?.wrapping_add(1),
                            )?;

                            break 'input;
                        }

                        let mut item_tapped = false;

                        for (item, counter) in [
                            (0usize, AppContext::SPEED_UP_PRESS),
                            (3usize, AppContext::CPU_PRESS),
                            (5usize, AppContext::SNIPER_PRESS),
                        ] {
                            let rect = AppContext::ITEM_RECTS.wrapping_add(item * 0x10);
                            let (rx, ry, rw, rh) = (
                                ctx.i32_at(rect)?,
                                ctx.i32_at(rect.wrapping_add(4))?,
                                ctx.i32_at(rect.wrapping_add(8))?,
                                ctx.i32_at(rect.wrapping_add(0xc))?,
                            );

                            if touch_released(ctx)? == 0 || !hit_test_rect(ctx, rx, ry, rw, rh)? {
                                continue;
                            }

                            let usable = if item == 0 {
                                powerup_available(ctx, 0)?
                            } else {
                                get_item_selected(ctx, item as i32)?
                            };

                            if usable {
                                ctx.set_i32_at(counter, ctx.i32_at(counter)?.wrapping_add(1))?;
                                item_tapped = true;

                                break;
                            }
                        }

                        if item_tapped {
                            play_sound(sound_manager(ctx)?, 0xb, None);

                            break 'input;
                        }

                        let (rx, ry, rw, rh) = (
                            ctx.i32_at(AppContext::CAT_GOD_BUTTON_RECT)?,
                            ctx.i32_at(AppContext::CAT_GOD_BUTTON_RECT.wrapping_add(4))?,
                            ctx.i32_at(AppContext::CAT_GOD_BUTTON_RECT.wrapping_add(8))?,
                            ctx.i32_at(AppContext::CAT_GOD_BUTTON_RECT.wrapping_add(0xc))?,
                        );

                        let cat_god_tapped = touch_released(ctx)? != 0
                            && hit_test_rect(ctx, rx, ry, rw, rh)?
                            && ctx.u8_at(AppContext::INPUT_BLOCKED)? == 0
                            && ctx.i32_at(AppContext::ENTRY_STAGE)? != 0x30
                            && ctx.i32_at(AppContext::CHAPTER_MODE)? != 3
                            && ctx.i32_at(AppContext::CHAPTER_MODE)? != 0x63
                            && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? == 0
                            && ctx.u8_at(AppContext::BATTLE_IS_INVASION)? == 0
                            && ctx.u8_at(AppContext::BATTLE_IS_Z_INVASION)? == 0
                            && !(get_map_type(ctx, 0)? == -7 && get_stage_index(ctx)? == 0x2f);

                        if cat_god_tapped {
                            if ctx.i32_at(AppContext::CAT_GOD_AVAILABLE)? > 0 {
                                ctx.set_i32_at(AppContext::UI_TAP_LOCKOUT, 0)?;
                                ctx.set_block_at::<1>(AppContext::INPUT_BLOCKED, [0])?;
                                play_sound(sound_manager(ctx)?, 0xb, None);
                                ctx.set_i32_at(
                                    AppContext::CAT_GOD_BUTTON_PRESS,
                                    ctx.i32_at(AppContext::CAT_GOD_BUTTON_PRESS)?
                                        .wrapping_add(1),
                                )?;
                                skip_banner = true;
                            }

                            break 'input;
                        }

                        if ctx.u8_at(AppContext::DECK_ROW_SWAPPING)? != 0
                            || ctx.i32_at(AppContext::SWIPE_VELOCITY)? != 0
                            || ctx.u8_at(AppContext::TOUCH_CAPTURED)? != 0
                            || ctx.u8_at(AppContext::DECK_SWIPE_LATCHED)? != 0
                        {
                            break 'input;
                        }

                        match ctx.i32_at(AppContext::DECK_ROW_SHOWN)? {
                            0 => ctx.set_i32_at(AppContext::DRAW_TEMP_0, 0)?,
                            1 => ctx.set_i32_at(AppContext::DRAW_TEMP_0, 5)?,
                            _ => {}
                        }

                        let mut two_lines = ctx.u8_at(AppContext::DECK_TWO_LINES)? != 0;
                        let mut button = 0usize;

                        loop {
                            let mut shift = 0i32;

                            if button <= 4 && two_lines {
                                let key = std_string_from_cstr(b"battle_slot_2lines_line");

                                shift = get_setting(&ctx.settings, &key, 0x5a)?.wrapping_neg();
                            }

                            let first = if ctx.u8_at(AppContext::DECK_TWO_LINES)? != 0 {
                                0
                            } else {
                                ctx.i32_at(AppContext::DRAW_TEMP_0)?
                            };
                            let index = (first as i64).wrapping_add(button as i64) as i32;

                            'press: {
                                if touch_is_down(ctx)? != 0 {
                                    let column = *ctx
                                        .deck_button_x
                                        .get(index as i64 as usize)
                                        .ok_or(Fault::index_out_of_range(index as i64, 10))? as f64;
                                    let x = ops::cvttsd2si(
                                        get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5
                                            + column,
                                    );
                                    let y = ctx
                                        .i32_at(AppContext::LETTERBOX_SHIFT)?
                                        .wrapping_add(ctx.deck_bar_base_y.wrapping_add(shift))
                                        .wrapping_sub(1);

                                    if hit_test_rect(ctx, x, y, 0x6e, 0x58)? {
                                        let pressed =
                                            AppContext::DECK_BUTTON_PRESSED.wrapping_add(button);

                                        if ctx.u8_at(pressed)? == 0 {
                                            play_sound(sound_manager(ctx)?, 0xa, None);
                                            ctx.set_block_at::<1>(pressed, [1])?;

                                            let unit_id = get_button_unit_id(ctx, 0, index)?;
                                            let form = get_button_unit_form(ctx, 0, index)?;

                                            if unit_id < 0 {
                                                break 'press;
                                            }

                                            if index == ctx.i32_at(AppContext::DECK_HOLD_SLOT)? {
                                                break 'press;
                                            }

                                            ctx.set_i32_at(AppContext::DECK_HOLD_SLOT, index)?;

                                            let name = get_cat_name(ctx, unit_id, form);
                                            let font = ctx.default_font.clone();
                                            let texture = get_text_texture(
                                                text_texture_cache(ctx)?,
                                                &name,
                                                &font,
                                                0x1e,
                                                1,
                                                0,
                                            );

                                            std_shared_ptr_texture_assign(
                                                &mut ctx.unit_info_texts[0],
                                                Some(texture),
                                            );
                                            std_shared_ptr_texture_reset(
                                                &mut ctx.unit_info_texts[1],
                                            );
                                            std_shared_ptr_texture_reset(
                                                &mut ctx.unit_info_texts[2],
                                            );
                                            std_shared_ptr_texture_reset(
                                                &mut ctx.unit_info_texts[3],
                                            );
                                            load_conjure_desc_textures(ctx, 1, unit_id, 0)?;
                                        } else if index == ctx.i32_at(AppContext::DECK_HOLD_SLOT)? {
                                            ctx.set_i32_at(
                                                AppContext::DECK_HOLD_FRAMES,
                                                ctx.i32_at(AppContext::DECK_HOLD_FRAMES)?
                                                    .wrapping_add(1),
                                            )?;

                                            if slot_desc_wait_elapsed(ctx)? {
                                                ctx.set_block_at::<1>(
                                                    AppContext::DECK_BUTTON_HELD,
                                                    [1],
                                                )?;
                                                ctx.set_i32_at(AppContext::DECK_HOLD_RELEASE, 3)?;
                                            }
                                        }

                                        break 'press;
                                    }
                                }

                                ctx.set_block_at::<1>(
                                    AppContext::DECK_BUTTON_PRESSED.wrapping_add(button),
                                    [0],
                                )?;
                            }

                            if touch_released(ctx)? != 0 {
                                let column = *ctx.deck_button_x.get(index as i64 as usize).ok_or(
                                    Fault::index_out_of_range(index as i64, 10),
                                )? as f64;
                                let x = ops::cvttsd2si(
                                    get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5
                                        + column,
                                );
                                let y = ctx
                                    .i32_at(AppContext::LETTERBOX_SHIFT)?
                                    .wrapping_add(shift.wrapping_add(ctx.deck_bar_base_y))
                                    .wrapping_sub(1);

                                if hit_test_rect(ctx, x, y, 0x6e, 0x58)?
                                    && ctx.u8_at(AppContext::INPUT_BLOCKED)? == 0
                                    && !slot_desc_wait_elapsed(ctx)?
                                {
                                    ctx.set_i32_at(AppContext::UI_TAP_LOCKOUT, 0)?;
                                    ctx.set_block_at::<1>(AppContext::INPUT_BLOCKED, [0])?;

                                    let counter = AppContext::DECK_PRESS.wrapping_add(button * 4);

                                    ctx.set_i32_at(counter, ctx.i32_at(counter)?.wrapping_add(1))?;

                                    break 'input;
                                }
                            }

                            button += 1;

                            let lines = ctx.u8_at(AppContext::DECK_TWO_LINES)?;

                            two_lines = lines != 0;

                            if button >= (lines as usize) * 5 + 5 {
                                break;
                            }
                        }
                    }
                }

                if skip_banner {
                    combo_banner_skip_all(ctx)?;
                }

                if get_powerup(ctx, 0)? {
                    if ctx.u8_at(AppContext::SPEED_UP_LATCH)? == 0 {
                        ctx.set_i32_at(AppContext::SPEED, 2)?;
                    } else {
                        let key = std_string_from_cstr(b"boost_speed");
                        let boost = get_setting(&ctx.settings, &key, 3)?;

                        ctx.set_i32_at(AppContext::SPEED, boost)?;
                    }
                } else {
                    ctx.set_i32_at(AppContext::SPEED, 1)?;
                }

                let cpu = get_powerup(ctx, 3)?;

                ctx.set_block_at::<1>(AppContext::CPU_ENABLED, [cpu as u8])?;

                if ctx.i32_at(AppContext::CAT_GOD_HEAL_PENDING)? == 1 {
                    let full = ctx.i32_at(AppContext::entity_field(0, 0, Entity::MAX_HP))?;

                    ctx.set_i32_at(AppContext::entity_field(0, 0, Entity::HP), full)?;

                    for slot in 1..0x33i32 {
                        if ctx.i32_at(AppContext::entity_field(0, slot, Entity::OCCUPANT))? == 0 {
                            continue;
                        }

                        if ctx.i32_at(AppContext::entity_field(0, slot, Entity::HP))? == 0 {
                            continue;
                        }

                        let max = ctx.i32_at(AppContext::entity_field(0, slot, Entity::MAX_HP))?;

                        ctx.set_i32_at(AppContext::entity_field(0, slot, Entity::HP), max)?;
                    }

                    ctx.set_i32_at(AppContext::CAT_GOD_HEAL_PENDING, 0)?;
                }

                if ctx.i32_at(AppContext::BABY_BOOM_ACTIVE)? == 1 {
                    ctx.set_i32_at(
                        AppContext::BABY_BOOM_FRAMES,
                        ctx.i32_at(AppContext::BABY_BOOM_FRAMES)?.wrapping_add(1),
                    )?;

                    for button in 0..10 {
                        set_deck_cooldown(ctx, AppContext::faction_flags(0), button, 0, 1)?;
                    }

                    if ctx.i32_at(AppContext::BABY_BOOM_FRAMES)? >= 0x708 {
                        ctx.set_i32_at(AppContext::BABY_BOOM_ACTIVE, 0)?;
                        ctx.set_i32_at(AppContext::BABY_BOOM_FRAMES, 0)?;
                    }
                }

                if get_battle_status(ctx)? != 0 {
                    ctx.set_i32_at(AppContext::SPEED, 1)?;
                    ctx.set_block_at::<1>(AppContext::CPU_ENABLED, [0])?;
                    ctx.set_i32_at(AppContext::BABY_BOOM_ACTIVE, 0)?;
                    ctx.set_i32_at(AppContext::BABY_BOOM_FRAMES, 0)?;
                }

                let swap_target = ctx.i32_at(AppContext::DECK_ROW_SWAP_TARGET)?;

                deck_row_swap_tick(ctx, swap_target)?;

                let zoom = ctx.i32_at(AppContext::CAMERA_ZOOM)?;
                let floor = ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?.wrapping_mul(0x64);

                let zoom = if zoom < floor {
                    ctx.set_i32_at(AppContext::CAMERA_ZOOM, floor)?;
                    floor
                } else if zoom <= get_max_zoom(ctx)? {
                    ctx.i32_at(AppContext::CAMERA_ZOOM)?
                } else {
                    let max = get_max_zoom(ctx)?;

                    ctx.set_i32_at(AppContext::CAMERA_ZOOM, max)?;
                    max
                };

                ctx.set_i32_at(AppContext::DRAW_TEMP_1, ops::div_100(zoom))?;

                let scale = zoom as f32 / 100.0 / 100.0;
                let right = -9600.0f32 / scale + ctx.i32_at(AppContext::STAGE_LENGTH)? as f32;
                let camera_x = ctx.i32_at(AppContext::CAMERA_X)?;

                let clamped = if camera_x as f32 > right {
                    Some(ops::cvttss2si(right))
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

                let mut field_full = true;

                for slot in 1..0x33i32 {
                    if ctx.i32_at(AppContext::entity_field(0, slot, Entity::OCCUPANT))? == 0 {
                        field_full = false;

                        break;
                    }
                }

                let refused = deploy_limit_reached(ctx)? || field_full;
                let map_id = get_global_map_id(ctx, 0)?;
                let cap = get_special_rule_params(ctx, &ctx.special_rules, map_id, 7)?.cloned();

                let capped = match cap {
                    Some(values) => {
                        let first = *values.first().ok_or(Fault::index_out_of_range(0, 0))?;

                        ctx.i32_at(AppContext::DEPLOY_LIMIT_TOTAL)? >= first
                    }
                    None => false,
                };

                if capped || refused {
                    ctx.set_i32_at(AppContext::DEPLOY_NOTICE_TIMER, 0)?;
                    ctx.set_i32_at(AppContext::DEPLOY_NOTICE_KIND, 1)?;
                } else {
                    let kind = ctx.i32_at(AppContext::DEPLOY_NOTICE_KIND)?;
                    let mut expired = true;

                    if kind & !1 == 2 {
                        let timer = ctx.i32_at(AppContext::DEPLOY_NOTICE_TIMER)?;

                        ctx.set_i32_at(AppContext::DEPLOY_NOTICE_TIMER, timer.wrapping_sub(1))?;
                        expired = timer <= 1;
                    }

                    if expired {
                        ctx.set_i32_at(AppContext::DEPLOY_NOTICE_KIND, 0)?;
                    }
                }

                let flash = ctx.i32_at(AppContext::DEPLOY_FULL_FLASH)?;

                if flash >= 0 {
                    ctx.set_i32_at(
                        AppContext::DEPLOY_FULL_FLASH,
                        if (flash as u32) < 0x32 {
                            flash.wrapping_add(1)
                        } else {
                            -1
                        },
                    )?;
                }

                demon_battle_banner_tick(ctx)?;

                if ctx.i32_at(AppContext::DEMON_BANNER_FRAME)? == -1 {
                    combo_banner_update(ctx)?;
                }

                for faction in 0..2i32 {
                    let wallet = AppContext::faction_flags(faction);

                    for button in 0..10usize {
                        if ctx.i32_at(
                            wallet
                                .wrapping_add(AppContext::WALLET_CONJURE_READY)
                                .wrapping_add(button * 4),
                        )? == 1
                            && ctx.i32_at(
                                wallet
                                    .wrapping_add(AppContext::WALLET_CONJURE_LOCKOUT)
                                    .wrapping_add(button * 4),
                            )? == 0
                        {
                            let timer = wallet
                                .wrapping_add(AppContext::WALLET_CONJURE_TIMER)
                                .wrapping_add(button * 4);

                            ctx.set_i32_at(timer, ctx.i32_at(timer)?.wrapping_add(1))?;
                        }
                    }
                }

                if is_score_stage(ctx.event_items.as_ref()) {
                    fever_fade_tick(ctx)?;
                }
                let mut step = 0i32;

                while step < ctx.i32_at(AppContext::SPEED)? {
                    'substep: {
                        if get_battle_status(ctx)? == 0 && is_score_stage(ctx.event_items.as_ref())
                        {
                            let points = ctx.event_items.as_ref().map_or(0, get_stage_score);

                            fever_tick(ctx, points)?;
                        }

                        let queued = ctx.deploy_queue.len() as i32;

                        if queued > 0 {
                            let mut index = queued as usize;

                            while index >= 1 {
                                let entry = *ctx.deploy_queue.get(index - 1).ok_or(
                                    Fault::index_out_of_range(index as i64 - 1, queued as i64),
                                )?;
                                let button = entry as u32 as i32;
                                let delay = (entry >> 32) as u32 as i32;

                                if delay > 0 {
                                    if let Some(record) = ctx.deploy_queue.get_mut(index - 1) {
                                        *record = (entry & 0xffff_ffff)
                                            | (((delay.wrapping_sub(1)) as u32 as u64) << 32);
                                    }
                                } else {
                                    let unit_id = get_button_unit_id(ctx, 0, button)?;
                                    let level = get_tech_level(
                                        ctx,
                                        AppContext::UNIT_LEVELS.wrapping_add(
                                            (unit_id as i64 as usize).wrapping_mul(8),
                                        ),
                                    )?;
                                    let counter = AppContext::faction_flags(0)
                                        .wrapping_add(AppContext::WALLET_DEPLOY_COUNTS)
                                        .wrapping_add((button as i64 as usize).wrapping_mul(4));

                                    ctx.set_i32_at(counter, ctx.i32_at(counter)?.wrapping_sub(1))?;

                                    let row = get_button_unit_row(ctx, 0, button)?;
                                    let z_row = get_button_unit_row(ctx, 0, button)?;
                                    let z_form = get_button_unit_form(ctx, 0, button)?;
                                    let z_min = ctx.i32_at(AppContext::cat_stat(
                                        z_row.wrapping_sub(2),
                                        z_form,
                                        CatStats::MINIMUM_Z_LAYER,
                                    ))?;
                                    let z_row = get_button_unit_row(ctx, 0, button)?;
                                    let z_form = get_button_unit_form(ctx, 0, button)?;
                                    let z_max = ctx.i32_at(AppContext::cat_stat(
                                        z_row.wrapping_sub(2),
                                        z_form,
                                        CatStats::MAXIMUM_Z_LAYER,
                                    ))?;
                                    let form = get_button_unit_form(ctx, 0, button)?;

                                    spawn_entity(ctx, 0, row, level, z_min, z_max, form, 0)?;
                                    ctx.set_i32_at(counter, ctx.i32_at(counter)?.wrapping_add(1))?;
                                    ctx.deploy_queue.remove(index - 1);
                                }

                                index -= 1;
                            }
                        }

                        for slot in 0..0x33 {
                            set_zkill_hit(ctx, 0, slot, 0)?;
                            set_barrier_state(ctx, 0, slot, 0)?;
                            set_shield_state(ctx, 0, slot, 0)?;
                        }

                        for slot in 0..0x33 {
                            set_zkill_hit(ctx, 1, slot, 0)?;
                            set_barrier_state(ctx, 1, slot, 0)?;
                            set_shield_state(ctx, 1, slot, 0)?;
                        }

                        'finish: {
                            if get_battle_status(ctx)? == 0 && is_scored_stage(ctx)? {
                                let elapsed = ctx.i32_at(AppContext::SCORE_ELAPSED)?;

                                if elapsed != 0x7fffffff {
                                    let elapsed = elapsed.wrapping_add(1);

                                    ctx.set_i32_at(AppContext::SCORE_ELAPSED, elapsed)?;

                                    if elapsed < get_score_time_limit(ctx)? {
                                        break 'finish;
                                    }

                                    if get_battle_status(ctx)? == 4 {
                                        break 'finish;
                                    }

                                    sound_manager(ctx)?.stop_audio(-1);

                                    let pays = scored_map_pays_money(ctx)?;

                                    play_sound(
                                        sound_manager(ctx)?,
                                        if pays { 0xbd } else { 0x39 },
                                        None,
                                    );
                                    vibration_reset(ctx)?;
                                    ctx.set_i32_at(AppContext::LOSE_TIP_SHOWN, 0)?;

                                    let saved = ctx.i32_at(AppContext::SAVED_MAP_TYPE)?;
                                    let map_type = validate_map_type(saved);
                                    let map_index = ctx.i32_at(AppContext::MAP_INDEX)?;
                                    let stage = ctx.i32_at(AppContext::STAGE_ROW)?;
                                    let star = ctx.i32_at(AppContext::STAR_LEVEL)?;
                                    let first_clear =
                                        get_stage_record(ctx, map_type, map_index, stage, star, 0)?
                                            <= 0;

                                    log_analytics_event(ctx, 0x1e, first_clear as i32, 0, 0, 0)?;

                                    let map_type =
                                        validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?);
                                    let map_index = ctx.i32_at(AppContext::MAP_INDEX)?;
                                    let stage = ctx.i32_at(AppContext::STAGE_ROW)?;
                                    let star = ctx.i32_at(AppContext::STAR_LEVEL)?;

                                    add_stage_record(ctx, map_type, map_index, stage, star, 1, 0)?;

                                    let map_type =
                                        validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?);
                                    let map_index = ctx.i32_at(AppContext::MAP_INDEX)?;
                                    let stage = ctx.i32_at(AppContext::STAGE_ROW)?;
                                    let star = ctx.i32_at(AppContext::STAR_LEVEL)?;

                                    if get_stage_record(ctx, map_type, map_index, stage, star, 0)?
                                        >= 0x2710
                                    {
                                        let map_type = validate_map_type(
                                            ctx.i32_at(AppContext::SAVED_MAP_TYPE)?,
                                        );
                                        let map_index = ctx.i32_at(AppContext::MAP_INDEX)?;
                                        let stage = ctx.i32_at(AppContext::STAGE_ROW)?;
                                        let star = ctx.i32_at(AppContext::STAR_LEVEL)?;

                                        set_stage_record(
                                            ctx, map_type, map_index, stage, star, 0x270f, 0,
                                        )?;
                                    }

                                    record_stage_played(ctx)?;
                                    set_battle_status(ctx, 4)?;

                                    let map_id = get_global_map_id(ctx, 0)?;

                                    mission_progress(ctx, 0, map_id, 1, 0, 0)?;

                                    let stage_key = map_id.wrapping_mul(0x64);
                                    let row =
                                        ctx.i32_at(AppContext::STAGE_ROW)?.wrapping_add(stage_key);

                                    mission_progress(ctx, 1, row, 1, 0, 0)?;
                                    mission_progress(ctx, 9, ops::div_1000(map_id), 1, 0, 0)?;

                                    let row =
                                        stage_key.wrapping_add(ctx.i32_at(AppContext::STAGE_ROW)?);
                                    let total = ctx.i32_at(AppContext::SCORE_TOTAL)?;

                                    mission_progress(ctx, 0x15, row, total, 0, 0)?;

                                    if get_map_type(ctx, 0)? == 4 {
                                        let map_index = ctx.i32_at(AppContext::MAP_INDEX)?;
                                        let best =
                                            entry_find_by_id(&ctx.ranking_entries, map_index);

                                        if best < ctx.i32_at(AppContext::SCORE_TOTAL)? {
                                            ctx.set_i32_at(AppContext::NEW_BEST_SCORE, 1)?;
                                        }
                                    } else {
                                        let map_index = ctx.i32_at(AppContext::MAP_INDEX)?;
                                        let stage_row = ctx.i32_at(AppContext::STAGE_ROW)?;
                                        let best = *std_map_int_int_subscript(
                                            std_map_int_map_int_subscript(
                                                &mut ctx.best_scores,
                                                &map_index,
                                            ),
                                            &stage_row,
                                        );
                                        let total = ctx.i32_at(AppContext::SCORE_TOTAL)?;

                                        if best < total {
                                            *std_map_int_int_subscript(
                                                std_map_int_map_int_subscript(
                                                    &mut ctx.best_scores,
                                                    &map_index,
                                                ),
                                                &stage_row,
                                            ) = total;
                                            ctx.set_i32_at(AppContext::NEW_BEST_SCORE, 1)?;
                                        }
                                    }

                                    battle_create_button(ctx)?;
                                    record_stage_lineup(ctx)?;

                                    break 'finish;
                                }
                            }

                            if get_battle_status(ctx)? != 0
                                || !is_score_stage(ctx.event_items.as_ref())
                            {
                                break 'finish;
                            }

                            if let Some(store) = ctx.event_items.as_mut() {
                                advance_point_decay(store);
                            }

                            if !ctx.event_items.as_ref().is_some_and(has_point_decay) {
                                break 'finish;
                            }

                            if !ctx.event_items.as_ref().is_some_and(point_decay_full) {
                                break 'finish;
                            }

                            if get_battle_status(ctx)? != 0 {
                                break 'finish;
                            }

                            ctx.set_block_at::<8>(AppContext::OUTRO_PHASE, [0; 8])?;
                            ctx.set_block_at::<16>(AppContext::BGM_SWITCH_STATE, BGM_SWITCH_RESET)?;
                            battle_init_win(ctx, 0)?;
                            battle_create_button(ctx)?;
                        }

                        let orbit = ctx.f32_at(AppContext::SNIPER_BOB_ANGLE)? + SNIPER_BOB_STEP;

                        ctx.set_f32_at(AppContext::SNIPER_BOB_ANGLE, orbit)?;

                        if get_battle_status(ctx)? == 3 {
                            break 'substep;
                        }

                        ctx.set_i32_at(
                            AppContext::BATTLE_FRAME_COUNTER,
                            ctx.i32_at(AppContext::BATTLE_FRAME_COUNTER)?
                                .wrapping_add(1),
                        )?;

                        if get_battle_status(ctx)? == 0 {
                            let played = ctx.i32_at(AppContext::PLAY_FRAMES)?;

                            ctx.set_i32_at(
                                AppContext::PLAY_FRAMES,
                                if played >= 0x752f {
                                    0x7530
                                } else {
                                    played.wrapping_add(1)
                                },
                            )?;
                        }

                        enemy_schedule_tick(ctx)?;
                        proc_update(ctx)?;
                        cat_update(ctx, 0)?;
                        sniper_update(ctx)?;
                        enemy_update(ctx, 1)?;

                        let mut shockwave_live = false;

                        for slot in 0..0x33 {
                            if is_boss(ctx, 1, slot)? && get_shockwave_counter(ctx, 1, slot)? == 0 {
                                shockwave_live = true;

                                break;
                            }
                        }

                        for slot in 1..0x33 {
                            if ctx.i32_at(AppContext::entity_field(0, slot, Entity::OCCUPANT))? == 0
                            {
                                continue;
                            }

                            if get_entity_state(ctx, 0, slot)? == 7 {
                                shockwave_live = true;

                                break;
                            }
                        }

                        'engage: for faction in 0..2i32 {
                            for slot in 1..0x33i32 {
                                if slot_occupied(ctx, faction, slot)? == 0 {
                                    continue;
                                }

                                if get_entity_state(ctx, faction, slot)? == 1
                                    || get_entity_state(ctx, faction, slot)? == 0
                                {
                                    if get_death_timer(ctx, faction, slot)? > 0 {
                                        add_death_timer(ctx, faction, slot, -1)?;

                                        if get_death_timer(ctx, faction, slot)? == 0 {
                                            set_entity_state(ctx, faction, slot, 4)?;
                                            set_entity_frame(ctx, faction, slot, 0)?;
                                        }
                                    } else if get_attacks_remaining(ctx, faction, slot)? == 0 {
                                        no_more_attacks(ctx, faction, slot)?;
                                    }

                                    if get_entity_state(ctx, faction, slot)? == 0
                                        && get_speed(ctx, faction, slot)? == 0
                                    {
                                        set_entity_state(ctx, faction, slot, 1)?;
                                    }
                                }

                                if get_entity_state(ctx, faction, slot)? != 0 {
                                    if get_entity_state(ctx, faction, slot)? != 0xc {
                                        continue;
                                    }

                                    if !check_collision(ctx, faction, slot, 0, 0)? {
                                        continue;
                                    }

                                    set_entity_state(ctx, faction, slot, 0xd)?;
                                    set_entity_frame(ctx, faction, slot, 0)?;

                                    if faction == 0 {
                                        continue 'engage;
                                    }

                                    break 'engage;
                                }

                                let mut target = 0i32;

                                while target < 0x33 {
                                    if check_collision(ctx, faction, slot, target, 0)? {
                                        break;
                                    }

                                    target += 1;
                                }

                                if target == 0x33 {
                                    continue;
                                }

                                let burrows = target != 0
                                    && is_zombie(ctx, faction, slot)?
                                    && get_burrow_count(ctx, faction, slot)? != 0
                                    && !(shockwave_live && is_boss(ctx, 1, slot)?);

                                if !burrows {
                                    set_entity_state(ctx, faction, slot, 1)?;
                                    set_entity_frame(ctx, faction, slot, 0)?;

                                    continue;
                                }

                                if get_burrow_count(ctx, faction, slot)? > 0 {
                                    add_burrow_count(ctx, faction, slot, -1)?;
                                }

                                set_entity_state(ctx, faction, slot, 0xb)?;
                                set_entity_frame(ctx, faction, slot, 0)?;

                                let x = get_pos_x(ctx, faction, slot)?;

                                set_burrow_start_x(ctx, faction, slot, x)?;
                            }
                        }

                        if ctx.i32_at(AppContext::CANNON_BLAST_ACTIVE)? == 1 {
                            ctx.set_i32_at(AppContext::CANNON_BLAST_ACTIVE, 0)?;
                        }

                        for slot in 1..0x33i32 {
                            if ctx.i32_at(AppContext::entity_field(0, slot, Entity::OCCUPANT))? == 0
                            {
                                continue;
                            }

                            if ctx.i32_at(AppContext::entity_field(0, slot, Entity::STATE))? == 1
                                && ctx.i32_at(AppContext::entity_field(
                                    0,
                                    slot,
                                    Entity::FREEZE_TIMER,
                                ))? <= 0
                            {
                                advance_animation_frame(ctx, 0, slot)?;

                                let mut engaged = false;
                                let mut next_state = None;

                                if get_attack_only(ctx, 0, slot)? {
                                    does_target(ctx, 0, slot, 0)?;

                                    if check_collision(ctx, 0, slot, 0, 0)? {
                                        engaged = true;

                                        if get_attack_cooldown(ctx, 0, slot)? == 0
                                            && get_attacks_remaining(ctx, 0, slot)? != 0
                                        {
                                            next_state = Some(2);
                                        }
                                    }

                                    if next_state.is_none() {
                                        for target in 1..0x33 {
                                            if !does_target(ctx, 0, slot, target)?
                                                && get_entity_base_idx(ctx)? != target
                                            {
                                                continue;
                                            }

                                            if !check_collision(ctx, 0, slot, target, 0)? {
                                                continue;
                                            }

                                            engaged = true;

                                            if get_attack_cooldown(ctx, 0, slot)? != 0 {
                                                continue;
                                            }

                                            if get_attacks_remaining(ctx, 0, slot)? == 0 {
                                                continue;
                                            }

                                            next_state = Some(2);

                                            break;
                                        }
                                    }

                                    if next_state.is_none() && !engaged {
                                        let mut touching = false;

                                        for target in 0..0x33 {
                                            if check_collision(ctx, 0, slot, target, 0)? {
                                                touching = true;

                                                break;
                                            }
                                        }

                                        if !touching {
                                            next_state = Some(0);
                                        }
                                    }
                                } else {
                                    for target in 0..0x33 {
                                        if !check_collision(ctx, 0, slot, target, 0)? {
                                            continue;
                                        }

                                        engaged = true;

                                        if get_attack_cooldown(ctx, 0, slot)? != 0 {
                                            continue;
                                        }

                                        if get_attacks_remaining(ctx, 0, slot)? == 0 {
                                            continue;
                                        }

                                        next_state = Some(2);

                                        break;
                                    }

                                    if next_state.is_none()
                                        && !engaged
                                        && get_speed(ctx, 0, slot)? > 0
                                    {
                                        next_state = Some(0);
                                    }
                                }

                                if let Some(state) = next_state {
                                    set_entity_state(ctx, 0, slot, state)?;
                                    set_entity_frame(ctx, 0, slot, 0)?;
                                }
                            }

                            if get_entity_state(ctx, 0, slot)? != 2 {
                                continue;
                            }

                            if get_freeze_timer(ctx, 0, slot)? > 0 {
                                continue;
                            }

                            advance_animation_frame(ctx, 0, slot)?;

                            if get_entity_frame(ctx, 0, slot)? == 0 {
                                if get_attacks_remaining(ctx, 0, slot)? == 0 {
                                    no_more_attacks(ctx, 0, slot)?;
                                } else {
                                    set_entity_state(ctx, 0, slot, 1)?;
                                }

                                continue;
                            }

                            if get_attack_count(ctx, 0, slot)? <= 0 {
                                continue;
                            }

                            let mut attack = 0i32;
                            let mut fired = None;

                            loop {
                                let frame = get_entity_frame(ctx, 0, slot)?;

                                if frame == get_attack_foreswing(ctx, 0, slot, attack)? {
                                    fired = Some(attack);

                                    break;
                                }

                                attack += 1;

                                if attack >= get_attack_count(ctx, 0, slot)? {
                                    break;
                                }
                            }

                            let Some(attack) = fired else {
                                continue;
                            };

                            if attack.wrapping_sub(get_attack_count(ctx, 0, slot)?) == -1 {
                                let interval = get_attack_interval(ctx, 0, slot)?;

                                set_attack_cooldown(ctx, 0, slot, interval)?;

                                if get_attacks_remaining(ctx, 0, slot)? > 0 {
                                    add_attacks_remaining(ctx, 0, slot, -1)?;
                                }
                            }

                            ctx.set_i32_at(AppContext::HIT_COUNT, 0)?;

                            if check_collision(ctx, 0, slot, 0, attack)? {
                                if get_attack_only(ctx, 0, slot)? {
                                    does_target(ctx, 0, slot, 0)?;
                                }

                                let count = ctx.i32_at(AppContext::HIT_COUNT)? as i64 as usize;

                                ctx.set_i32_at(
                                    AppContext::HIT_LIST.wrapping_add(count.wrapping_mul(8)),
                                    0,
                                )?;

                                let distance = ctx
                                    .i32_at(AppContext::entity_field(1, 0, Entity::POS_X))?
                                    .wrapping_sub(ctx.i32_at(AppContext::entity_field(
                                        1,
                                        0,
                                        Entity::HITBOX_POS,
                                    ))?);
                                let count = ctx.i32_at(AppContext::HIT_COUNT)? as i64 as usize;

                                ctx.set_i32_at(
                                    AppContext::HIT_LIST
                                        .wrapping_add(count.wrapping_mul(8))
                                        .wrapping_add(4),
                                    distance,
                                )?;
                                ctx.set_i32_at(
                                    AppContext::HIT_COUNT,
                                    (count as i32).wrapping_add(1),
                                )?;
                            }

                            for target in 1..0x33i32 {
                                if !check_collision(ctx, 0, slot, target, attack)? {
                                    continue;
                                }

                                if get_attack_only(ctx, 0, slot)?
                                    && !does_target(ctx, 0, slot, target)?
                                    && get_entity_base_idx(ctx)? as u32 as i32 != target
                                {
                                    continue;
                                }

                                let count = ctx.i32_at(AppContext::HIT_COUNT)? as i64 as usize;

                                ctx.set_i32_at(
                                    AppContext::HIT_LIST.wrapping_add(count.wrapping_mul(8)),
                                    target,
                                )?;

                                let distance = ctx
                                    .i32_at(AppContext::entity_field(1, target, Entity::POS_X))?
                                    .wrapping_sub(ctx.i32_at(AppContext::entity_field(
                                        1,
                                        target,
                                        Entity::HITBOX_POS,
                                    ))?);
                                let count = ctx.i32_at(AppContext::HIT_COUNT)? as i64 as usize;

                                ctx.set_i32_at(
                                    AppContext::HIT_LIST
                                        .wrapping_add(count.wrapping_mul(8))
                                        .wrapping_add(4),
                                    distance,
                                )?;
                                ctx.set_i32_at(
                                    AppContext::HIT_COUNT,
                                    (count as i32).wrapping_add(1),
                                )?;
                            }

                            if ctx.i32_at(AppContext::HIT_COUNT)? <= 0 {
                                continue;
                            }

                            if get_area_attack(ctx, 0, slot)? {
                                let mut targets = Vec::new();
                                let mut index = 0i64;

                                while index < ctx.i32_at(AppContext::HIT_COUNT)? as i64 {
                                    targets.push(
                                        ctx.i32_at(
                                            AppContext::HIT_LIST
                                                .wrapping_add((index as usize).wrapping_mul(8)),
                                        )?,
                                    );
                                    index += 1;
                                }

                                cat_hit_executor(ctx, slot, &targets, attack)?;

                                continue;
                            }

                            ctx.zero(AppContext::PROC_ROLLS, 0x30)?;

                            let count = ctx.i32_at(AppContext::HIT_COUNT)?;

                            if count >= 2 {
                                let mut first = 0i64;

                                while first < ctx.i32_at(AppContext::HIT_COUNT)? as i64 - 1 {
                                    let entry = AppContext::HIT_LIST
                                        .wrapping_add((first as usize).wrapping_mul(8));
                                    let mut best = first;
                                    let mut best_distance = ctx.i32_at(entry.wrapping_add(4))?;
                                    let mut other = first + 1;

                                    while other < ctx.i32_at(AppContext::HIT_COUNT)? as i64 {
                                        let distance = ctx.i32_at(
                                            AppContext::HIT_LIST
                                                .wrapping_add((other as usize).wrapping_mul(8))
                                                .wrapping_add(4),
                                        )?;

                                        if distance > best_distance {
                                            best = other;
                                            best_distance = distance;
                                        }

                                        other += 1;
                                    }

                                    let winner = AppContext::HIT_LIST.wrapping_add(
                                        (best as i32 as i64 as usize).wrapping_mul(8),
                                    );

                                    ctx.set_i32_at(AppContext::HIT_SWAP, ctx.i32_at(entry)?)?;
                                    ctx.set_i32_at(
                                        AppContext::HIT_SWAP.wrapping_add(4),
                                        ctx.i32_at(entry.wrapping_add(4))?,
                                    )?;
                                    ctx.set_block_at::<8>(entry, ctx.block_at::<8>(winner)?)?;
                                    ctx.set_block_at::<8>(
                                        winner,
                                        ctx.block_at::<8>(AppContext::HIT_SWAP)?,
                                    )?;
                                    first += 1;
                                }
                            }

                            let count = ctx.i32_at(AppContext::HIT_COUNT)?;
                            let mut ties = 0i32;

                            if count > 0 {
                                let lead = ctx.i32_at(AppContext::HIT_LIST.wrapping_add(4))?;

                                while ties < count
                                    && ctx.i32_at(
                                        AppContext::HIT_LIST
                                            .wrapping_add((ties as usize).wrapping_mul(8))
                                            .wrapping_add(4),
                                    )? == lead
                                {
                                    ties += 1;
                                }
                            }

                            let pick = call_rng(ctx, ties);
                            let target = ctx.i32_at(
                                AppContext::HIT_LIST
                                    .wrapping_add((pick as i64 as usize).wrapping_mul(8)),
                            )?;

                            cat_hit_executor(ctx, slot, &[target], attack)?;
                        }

                        for slot in 1..0x33i32 {
                            if ctx.i32_at(AppContext::entity_field(1, slot, Entity::OCCUPANT))? == 0
                            {
                                continue;
                            }

                            if get_entity_state(ctx, 1, slot)? == 1
                                && get_freeze_timer(ctx, 1, slot)? <= 0
                            {
                                advance_animation_frame(ctx, 1, slot)?;

                                let mut engaged = false;
                                let mut next_state = None;

                                if check_collision(ctx, 1, slot, 0, 0)? {
                                    engaged = true;

                                    if get_attack_cooldown(ctx, 1, slot)? == 0
                                        && get_attacks_remaining(ctx, 1, slot)? != 0
                                    {
                                        next_state = Some(2);
                                    }
                                }

                                if next_state.is_none() {
                                    for target in 1..0x33 {
                                        if !check_collision(ctx, 1, slot, target, 0)? {
                                            continue;
                                        }

                                        if is_zombie(ctx, 1, slot)?
                                            && get_burrow_count(ctx, 1, slot)? != 0
                                            && !(shockwave_live && is_boss(ctx, 1, slot)?)
                                        {
                                            if get_burrow_count(ctx, 1, slot)? > 0 {
                                                add_burrow_count(ctx, 1, slot, -1)?;
                                            }

                                            set_entity_state(ctx, 1, slot, 0xb)?;
                                            set_entity_frame(ctx, 1, slot, 0)?;

                                            let x = get_pos_x(ctx, 1, slot)?;

                                            set_burrow_start_x(ctx, 1, slot, x)?;
                                            next_state = Some(-1);

                                            break;
                                        }

                                        engaged = true;

                                        if get_attack_cooldown(ctx, 1, slot)? != 0 {
                                            continue;
                                        }

                                        if get_attacks_remaining(ctx, 1, slot)? == 0 {
                                            continue;
                                        }

                                        next_state = Some(2);

                                        break;
                                    }
                                }

                                if next_state.is_none() && !engaged && get_speed(ctx, 1, slot)? > 0
                                {
                                    next_state = Some(0);
                                }

                                if let Some(state) = next_state.filter(|state| *state >= 0) {
                                    set_entity_state(ctx, 1, slot, state)?;
                                    set_entity_frame(ctx, 1, slot, 0)?;
                                }
                            }

                            if get_entity_state(ctx, 1, slot)? != 2 {
                                continue;
                            }

                            if get_freeze_timer(ctx, 1, slot)? > 0 {
                                continue;
                            }

                            advance_animation_frame(ctx, 1, slot)?;

                            if get_entity_frame(ctx, 1, slot)? == 0 {
                                if get_attacks_remaining(ctx, 1, slot)? == 0 {
                                    no_more_attacks(ctx, 1, slot)?;
                                } else {
                                    set_entity_state(ctx, 1, slot, 1)?;
                                }

                                continue;
                            }

                            if get_attack_count(ctx, 1, slot)? <= 0 {
                                continue;
                            }

                            let mut attack = 0i32;

                            loop {
                                let frame = get_entity_frame(ctx, 1, slot)?;

                                'fire: {
                                    if frame != get_attack_foreswing(ctx, 1, slot, attack)? {
                                        break 'fire;
                                    }

                                    if attack == get_attack_count(ctx, 1, slot)?.wrapping_sub(1) {
                                        let interval = get_attack_interval(ctx, 1, slot)?;

                                        set_attack_cooldown(ctx, 1, slot, interval)?;

                                        if get_attacks_remaining(ctx, 1, slot)? > 0 {
                                            add_attacks_remaining(ctx, 1, slot, -1)?;
                                        }
                                    }

                                    ctx.set_i32_at(AppContext::HIT_COUNT, 0)?;

                                    for target in 0..0x33i32 {
                                        if !check_collision(ctx, 1, slot, target, attack)? {
                                            continue;
                                        }

                                        let count =
                                            ctx.i32_at(AppContext::HIT_COUNT)? as i64 as usize;

                                        ctx.set_i32_at(
                                            AppContext::HIT_LIST
                                                .wrapping_add(count.wrapping_mul(8)),
                                            target,
                                        )?;

                                        let distance = ctx
                                            .i32_at(AppContext::entity_field(
                                                0,
                                                target,
                                                Entity::HITBOX_POS,
                                            ))?
                                            .wrapping_add(ctx.i32_at(AppContext::entity_field(
                                                0,
                                                target,
                                                Entity::POS_X,
                                            ))?);
                                        let count =
                                            ctx.i32_at(AppContext::HIT_COUNT)? as i64 as usize;

                                        ctx.set_i32_at(
                                            AppContext::HIT_LIST
                                                .wrapping_add(count.wrapping_mul(8))
                                                .wrapping_add(4),
                                            distance,
                                        )?;
                                        ctx.set_i32_at(
                                            AppContext::HIT_COUNT,
                                            (count as i32).wrapping_add(1),
                                        )?;
                                    }

                                    if ctx.i32_at(AppContext::HIT_COUNT)? <= 0 {
                                        break 'fire;
                                    }

                                    if get_area_attack(ctx, 1, slot)? {
                                        let mut targets = Vec::new();
                                        let mut index = 0i64;

                                        while index < ctx.i32_at(AppContext::HIT_COUNT)? as i64 {
                                            targets.push(
                                                ctx.i32_at(AppContext::HIT_LIST.wrapping_add(
                                                    (index as usize).wrapping_mul(8),
                                                ))?,
                                            );
                                            index += 1;
                                        }

                                        enemy_hit_executor(ctx, slot, &targets, attack)?;

                                        break 'fire;
                                    }

                                    ctx.zero(AppContext::PROC_ROLLS, 0x30)?;

                                    let count = ctx.i32_at(AppContext::HIT_COUNT)?;

                                    if count >= 2 {
                                        let mut first = 0i64;

                                        while first < ctx.i32_at(AppContext::HIT_COUNT)? as i64 - 1
                                        {
                                            let entry = AppContext::HIT_LIST
                                                .wrapping_add((first as usize).wrapping_mul(8));
                                            let mut best = first;
                                            let mut best_distance =
                                                ctx.i32_at(entry.wrapping_add(4))?;
                                            let mut other = first + 1;

                                            while other < ctx.i32_at(AppContext::HIT_COUNT)? as i64
                                            {
                                                let distance = ctx.i32_at(
                                                    AppContext::HIT_LIST
                                                        .wrapping_add(
                                                            (other as usize).wrapping_mul(8),
                                                        )
                                                        .wrapping_add(4),
                                                )?;

                                                if distance < best_distance {
                                                    best = other;
                                                    best_distance = distance;
                                                }

                                                other += 1;
                                            }

                                            let winner = AppContext::HIT_LIST.wrapping_add(
                                                (best as i32 as i64 as usize).wrapping_mul(8),
                                            );

                                            ctx.set_i32_at(
                                                AppContext::HIT_SWAP,
                                                ctx.i32_at(entry)?,
                                            )?;
                                            ctx.set_i32_at(
                                                AppContext::HIT_SWAP.wrapping_add(4),
                                                ctx.i32_at(entry.wrapping_add(4))?,
                                            )?;
                                            ctx.set_block_at::<8>(
                                                entry,
                                                ctx.block_at::<8>(winner)?,
                                            )?;
                                            ctx.set_block_at::<8>(
                                                winner,
                                                ctx.block_at::<8>(AppContext::HIT_SWAP)?,
                                            )?;
                                            first += 1;
                                        }
                                    }

                                    let count = ctx.i32_at(AppContext::HIT_COUNT)?;
                                    let mut ties = 0i32;

                                    if count > 0 {
                                        let lead =
                                            ctx.i32_at(AppContext::HIT_LIST.wrapping_add(4))?;

                                        while ties < count
                                            && ctx.i32_at(
                                                AppContext::HIT_LIST
                                                    .wrapping_add((ties as usize).wrapping_mul(8))
                                                    .wrapping_add(4),
                                            )? == lead
                                        {
                                            ties += 1;
                                        }
                                    }

                                    let pick = call_rng(ctx, ties);
                                    let target = ctx
                                        .i32_at(AppContext::HIT_LIST.wrapping_add(
                                            (pick as i64 as usize).wrapping_mul(8),
                                        ))?;

                                    enemy_hit_executor(ctx, slot, &[target], attack)?;
                                }

                                attack += 1;

                                if attack >= get_attack_count(ctx, 1, slot)? {
                                    break;
                                }
                            }
                        }

                        for slot in 0..0x33i32 {
                            if !is_boss(ctx, 1, slot)? {
                                continue;
                            }

                            if get_shockwave_counter(ctx, 1, slot)? == 0 {
                                if get_boss_type(ctx, 1, slot)? == 2 {
                                    base_shake_start(ctx, 1);
                                }

                                play_sound(sound_manager(ctx)?, 0x2d, None);

                                for target in 1..0x33i32 {
                                    if !is_touchable_thunk(ctx, 0, target, -1)? {
                                        continue;
                                    }

                                    if get_boss_wave_immune(ctx, 0, target)? {
                                        continue;
                                    }

                                    set_entity_state(ctx, 0, target, 7)?;
                                    set_entity_frame(ctx, 0, target, 0)?;
                                }
                            }

                            add_shockwave_counter(ctx, 1, slot, 1)?;

                            let counter = get_shockwave_counter(ctx, 1, slot)?;

                            if counter >= maanim_get_max_keyframe(&ctx.boss_shockwave_anim)? {
                                let length = maanim_get_max_keyframe(&ctx.boss_shockwave_anim)?;

                                set_shockwave_counter(ctx, 1, slot, length)?;
                            }
                        }

                        wave_update(ctx)?;
                        wave_apply_hits(ctx)?;
                        surge_update(ctx)?;
                        counter_surge_update(ctx)?;
                        explosion_update(ctx)?;
                        cannon_attack(ctx, 0)?;
                        cannon_attack(ctx, 1)?;

                        let mut shield_hits: [BTreeMap<i32, u8>; 2] =
                            [BTreeMap::new(), BTreeMap::new()];
                        let base_slot = if has_castle_enemy(ctx)? {
                            get_entity_base_idx(ctx)? as u32 as i64
                        } else {
                            0
                        };

                        for faction in 0..2i32 {
                            for slot in 0..0x33i32 {
                                if get_barrier_vfx_active(ctx, faction, slot)? {
                                    add_barrier_vfx_frame(ctx, faction, slot, 1)?;

                                    let frame = get_barrier_vfx_frame(ctx, faction, slot)?;

                                    if frame >= get_anim_len(&ctx.barrier_anims[0])? {
                                        set_barrier_vfx_active(ctx, faction, slot, 0)?;
                                    }
                                }

                                if get_shield_vfx(ctx, faction, slot)? != 0 {
                                    add_shield_vfx_frame(ctx, faction, slot, 1)?;

                                    let frame = get_shield_vfx_frame(ctx, faction, slot)?;
                                    let kind = get_shield_vfx(ctx, faction, slot)?.wrapping_sub(1);
                                    let anim = ctx.shield_anims.get(kind as i64 as usize).ok_or(
                                        Fault::index_out_of_range(kind as i64, 5),
                                    )?;

                                    if frame >= get_anim_len(anim)? {
                                        set_shield_vfx(ctx, faction, slot, 0)?;
                                    }
                                }

                                if faction == 1 {
                                    if ctx.metal_killer_map.contains_key(&slot) {
                                        let key =
                                            std_string_from_cstr(b"battle_metal_killer_hp_type");
                                        let mut remaining =
                                            if get_setting(&ctx.settings, &key, 0)? != 0 {
                                                get_hp(ctx, 1, slot)?
                                            } else {
                                                get_max_hp(ctx, 1, slot)?
                                            };
                                        let percents =
                                            ctx.metal_killer_map.entry(slot).or_default().clone();

                                        for percent in percents {
                                            let damage = max_i32(
                                                ops::div_100(percent.wrapping_mul(remaining)),
                                                1,
                                            );

                                            attack_dmg_dispatch(ctx, 1, slot, damage)?;
                                            remaining = remaining.wrapping_sub(damage);

                                            if remaining <= 0 {
                                                remaining = 0;
                                            }
                                        }
                                    }

                                    if slot as i64 == base_slot
                                        && base_resists_one_shot(ctx)?
                                        && get_took_damage(ctx, 1, base_slot as i32)?
                                        && get_hp(ctx, 1, base_slot as i32)? >= 2
                                        && get_hp(ctx, 1, base_slot as i32)?
                                            <= get_frame_damage(ctx, 1, base_slot as i32)?
                                    {
                                        let hp = get_hp(ctx, 1, base_slot as i32)?;

                                        set_frame_damage(
                                            ctx,
                                            1,
                                            base_slot as i32,
                                            hp.wrapping_sub(1),
                                        )?;
                                    }
                                }

                                if slot_occupied(ctx, faction, slot)? != 2 {
                                    continue;
                                }

                                if !get_took_damage(ctx, faction, slot)? {
                                    continue;
                                }

                                if get_barrier_hp(ctx, faction, slot)? > 0
                                    || get_shield_hp(ctx, faction, slot)? > 0
                                {
                                    if get_shield_hp(ctx, faction, slot)? > 0 {
                                        shield_hits[faction as usize].insert(slot, 1);

                                        if get_shield_state(ctx, faction, slot)? == 0 {
                                            let left = max_i32(
                                                get_shield_hp(ctx, faction, slot)?.wrapping_sub(
                                                    get_frame_damage(ctx, faction, slot)?,
                                                ),
                                                0,
                                            );

                                            set_shield_hp(ctx, faction, slot, left)?;

                                            if left <= 0 {
                                                set_shield_state(ctx, faction, slot, 1)?;
                                            }
                                        }
                                    }

                                    if get_barrier_state(ctx, faction, slot)? != 0 {
                                        set_barrier_hp(ctx, faction, slot, 0)?;
                                    }

                                    if get_shield_state(ctx, faction, slot)? != 0 {
                                        set_shield_hp(ctx, faction, slot, 0)?;
                                    }

                                    if (get_barrier_hp(ctx, faction, slot)? > 0
                                        && get_barrier_state(ctx, faction, slot)? == 0)
                                        || get_barrier_state(ctx, faction, slot)? == 1
                                        || (get_shield_hp(ctx, faction, slot)? > 0
                                            && get_shield_state(ctx, faction, slot)? == 0)
                                        || get_shield_state(ctx, faction, slot)? == 1
                                    {
                                        set_frame_damage(ctx, faction, slot, 0)?;
                                        set_drain_pct(ctx, faction, slot, 0)?;

                                        let freeze = get_prev_freeze_timer(ctx, faction, slot)?;

                                        set_freeze_timer(ctx, faction, slot, freeze)?;

                                        let slow = get_prev_slow_timer(ctx, faction, slot)?;

                                        set_slow_timer(ctx, faction, slot, slow)?;

                                        let weaken = get_prev_weaken_timer(ctx, faction, slot)?;

                                        set_weaken_timer(ctx, faction, slot, weaken)?;

                                        let curse = get_prev_curse_timer(ctx, faction, slot)?;

                                        set_curse_timer(ctx, faction, slot, curse)?;
                                        set_score_hit_mask(ctx, faction, slot, 0)?;
                                    }

                                    if get_barrier_hp(ctx, faction, slot)? > 0
                                        || get_barrier_state(ctx, faction, slot)? == 1
                                    {
                                        set_crit_vfx(ctx, faction, slot, 0)?;
                                        set_savage_blow_vfx(ctx, faction, slot, 0)?;
                                        set_toxic_vfx(ctx, faction, slot, 0)?;
                                        set_metal_killer_vfx(ctx, faction, slot, 0)?;
                                    }

                                    if get_barrier_hp(ctx, faction, slot)? > 0
                                        && get_barrier_state(ctx, faction, slot)? == 0
                                    {
                                        if !get_barrier_vfx_active(ctx, faction, slot)? {
                                            set_barrier_vfx_active(ctx, faction, slot, 1)?;
                                        }

                                        set_barrier_vfx_frame(ctx, faction, slot, 0)?;
                                        play_sound_in_battle(ctx, 0x47)?;
                                    } else if get_barrier_state(ctx, faction, slot)? == 1
                                        || get_barrier_state(ctx, faction, slot)? == 2
                                    {
                                        set_barrier_vfx_active(ctx, faction, slot, 0)?;

                                        if get_barrier_state(ctx, faction, slot)? == 1 {
                                            play_sound_in_battle(ctx, 0x48)?;
                                        } else if get_barrier_state(ctx, faction, slot)? == 2 {
                                            play_sound_in_battle(ctx, 0x46)?;
                                        }

                                        let mut free = None;

                                        for record in 0..0x1eusize {
                                            if ctx.u8_at(AppContext::BARRIER_VFX.wrapping_add(
                                                record * AppContext::BARRIER_VFX_STRIDE,
                                            ))? == 0
                                            {
                                                free = Some(record);

                                                break;
                                            }
                                        }

                                        if let Some(record) = free {
                                            let vfx = AppContext::BARRIER_VFX.wrapping_add(
                                                record * AppContext::BARRIER_VFX_STRIDE,
                                            );

                                            clear_barrier_vfx_slot(ctx, vfx)?;
                                            ctx.set_block_at::<1>(vfx, [1])?;
                                            ctx.set_i32_at(
                                                vfx.wrapping_add(VfxSlot::POS_X),
                                                ctx.i32_at(AppContext::entity_field(
                                                    faction,
                                                    slot,
                                                    Entity::POS_X,
                                                ))?,
                                            )?;
                                            ctx.set_i32_at(
                                                vfx.wrapping_add(VfxSlot::POS_Y),
                                                ctx.i32_at(AppContext::entity_field(
                                                    faction,
                                                    slot,
                                                    Entity::POS_Y,
                                                ))?,
                                            )?;

                                            let broken =
                                                get_barrier_state(ctx, faction, slot)? == 2;

                                            ctx.set_block_at::<1>(
                                                vfx.wrapping_add(VfxSlot::BROKEN),
                                                [broken as u8],
                                            )?;
                                        }
                                    }

                                    if get_shield_hp(ctx, faction, slot)? > 0
                                        && get_shield_state(ctx, faction, slot)? == 0
                                    {
                                        if get_shield_vfx(ctx, faction, slot)? == 0 {
                                            let low = get_shield_hp(ctx, faction, slot)?
                                                < ops::div_2(get_shield_max(
                                                    ctx, faction, slot,
                                                )?);

                                            set_shield_vfx(ctx, faction, slot, low as i32 + 1)?;
                                        }

                                        set_shield_vfx_frame(ctx, faction, slot, 0)?;
                                        play_sound_in_battle(ctx, 0x88)?;
                                    } else if get_shield_state(ctx, faction, slot)? == 1
                                        || get_shield_state(ctx, faction, slot)? == 2
                                    {
                                        set_shield_vfx(ctx, faction, slot, 0)?;

                                        if get_shield_state(ctx, faction, slot)? == 1 {
                                            play_sound_in_battle(ctx, 0x89)?;
                                        } else if get_shield_state(ctx, faction, slot)? == 2 {
                                            play_sound_in_battle(ctx, 0x8b)?;
                                        }

                                        let mut free = None;

                                        for record in 0..0x1eusize {
                                            if ctx.u8_at(AppContext::SHIELD_VFX.wrapping_add(
                                                record * AppContext::SHIELD_VFX_STRIDE,
                                            ))? == 0
                                            {
                                                free = Some(record);

                                                break;
                                            }
                                        }

                                        if let Some(record) = free {
                                            let vfx = AppContext::SHIELD_VFX.wrapping_add(
                                                record * AppContext::SHIELD_VFX_STRIDE,
                                            );

                                            clear_shield_vfx_slot(ctx, vfx)?;
                                            ctx.set_block_at::<1>(vfx, [1])?;
                                            ctx.set_i32_at(
                                                vfx.wrapping_add(VfxSlot::POS_X),
                                                ctx.i32_at(AppContext::entity_field(
                                                    faction,
                                                    slot,
                                                    Entity::POS_X,
                                                ))?,
                                            )?;
                                            ctx.set_i32_at(
                                                vfx.wrapping_add(VfxSlot::POS_Y),
                                                ctx.i32_at(AppContext::entity_field(
                                                    faction,
                                                    slot,
                                                    Entity::POS_Y,
                                                ))?
                                                .wrapping_add(-0x1a4),
                                            )?;

                                            let broken = get_shield_state(ctx, faction, slot)? == 2;

                                            ctx.set_block_at::<1>(
                                                vfx.wrapping_add(VfxSlot::BROKEN),
                                                [broken as u8],
                                            )?;
                                        }
                                    }
                                }

                                if battle_not_finishing(ctx)? {
                                    let mask = get_score_hit_mask(ctx, faction, slot)?;

                                    ctx.set_i32_at(
                                        AppContext::SCORE_TOTAL,
                                        ctx.i32_at(AppContext::SCORE_TOTAL)?.wrapping_add(mask),
                                    )?;
                                    set_score_hit_mask(ctx, faction, slot, 0)?;
                                }
                            }
                        }

                        let mut event = 0usize;

                        while event < ctx.counter_surge_events.len() {
                            if ctx.counter_surge_events[event].state == 0 {
                                let (faction, slot) = (
                                    ctx.counter_surge_events[event].faction,
                                    ctx.counter_surge_events[event].slot,
                                );
                                let hurt = get_frame_damage(ctx, faction, slot)? > 0;

                                ctx.counter_surge_events[event].state = hurt as i32 + 1;
                            }

                            event += 1;
                        }

                        let mut entry = 0usize;

                        while entry < ctx.stage_enemies.len() {
                            let row = ctx.stage_enemies[entry];

                            'trigger: {
                                if has_castle_enemy(ctx)?
                                    && ctx.i32_at(AppContext::enemy_stat(
                                        get_castle_enemy_row(ctx)?.wrapping_sub(2),
                                        EnemyStats::TRAIT_DOJO,
                                    ))? != 0
                                {
                                    let base = get_entity_base_idx(ctx)?;

                                    if get_total_damage_taken(ctx, 1, base)?
                                        < stage_entry_base_trigger(&row)
                                    {
                                        break 'trigger;
                                    }
                                } else {
                                    let hp = get_hp(ctx, 1, 0)?;
                                    let damage = get_frame_damage(ctx, 1, 0)?;
                                    let max =
                                        ctx.i32_at(AppContext::entity_field(1, 0, Entity::MAX_HP))?;

                                    if hp.wrapping_sub(damage)
                                        > ops::div_100(
                                            stage_entry_base_trigger(&row).wrapping_mul(max),
                                        )
                                    {
                                        break 'trigger;
                                    }
                                }

                                if stage_entry_count(&row) != 0 {
                                    let spawned =
                                        ctx.spawn_states.get(entry).map(|state| state[1]).ok_or(
                                            Fault::index_out_of_range(entry as i64, ctx.spawn_states.len() as i64),
                                        )?;

                                    if spawned >= stage_entry_count(&row) {
                                        break 'trigger;
                                    }
                                }

                                if stage_entry_is_boss(&row)
                                    && ctx.i32_at(AppContext::entity_field(1, 0, Entity::HP))? >= 2
                                {
                                    ctx.set_block_at::<1>(AppContext::BASE_KILL_BLOCKED, [1])?;
                                }
                            }

                            entry += 1;
                        }

                        if has_castle_enemy(ctx)? {
                            let base = get_entity_base_idx(ctx)?;
                            let hp = get_hp(ctx, 1, base)?;

                            set_hp(ctx, 1, 0, hp)?;

                            let max = get_max_hp(ctx, 1, base)?;

                            set_max_hp(ctx, 1, 0, max)?;

                            let damage = get_frame_damage(ctx, 1, base)?;

                            set_frame_damage(ctx, 1, 0, damage)?;
                        }

                        let mut hit_sound = 0i32;

                        for faction in 0..2i32 {
                            let first = faction == 0;
                            let mut red_gauge = [0i32; 10];
                            let wallet = AppContext::faction_flags(faction);
                            let debris_ring = AppContext::CAT_DEBRIS
                                .wrapping_add((faction as usize).wrapping_mul(0x380));
                            let mut crit_index = 0usize;

                            for slot in 0..0x33i32 {
                                if ctx.i32_at(AppContext::entity_field(
                                    faction,
                                    slot,
                                    Entity::OCCUPANT,
                                ))? == 0
                                {
                                    continue;
                                }

                                if get_frame_damage(ctx, faction, slot)? != 0 {
                                    let exempt = has_castle_enemy(ctx)?
                                        && !first
                                        && (slot == 0
                                            || get_entity_base_idx(ctx)? as u32 as i32 == slot)
                                        && get_trait_dojo(ctx, 1, get_entity_base_idx(ctx)?)?;

                                    if !exempt {
                                        let is_base = if slot == 0 || first {
                                            slot == 0
                                        } else {
                                            get_entity_base_idx(ctx)? as u32 as i32 == slot
                                        };

                                        latch_battle_event(
                                            &mut ctx.battle_event_latch,
                                            faction,
                                            is_base as i32 | 2,
                                        );
                                    }
                                }

                                if get_frame_damage(ctx, faction, slot)? == 0
                                    && *shield_hits[faction as usize].entry(slot).or_insert(0) == 0
                                {
                                    continue;
                                }

                                if !(slot != 0 || first) && has_castle_enemy(ctx)? {
                                    continue;
                                }

                                let entity = AppContext::entity_field(faction, slot, 0);
                                let base = AppContext::entity_field(faction, 0, 0);

                                if get_crit_vfx(ctx, faction, slot)?
                                    || get_savage_blow_vfx(ctx, faction, slot)?
                                    || get_toxic_vfx(ctx, faction, slot)?
                                    || get_metal_killer_vfx(ctx, faction, slot)?
                                {
                                    if get_crit_vfx(ctx, faction, slot)? {
                                        crit_index = 0;

                                        for record in 0..200usize {
                                            let vfx = AppContext::CRIT_VFX
                                                .wrapping_add(record * AppContext::CRIT_VFX_STRIDE);

                                            if ctx.u8_at(vfx)? == 0 {
                                                ctx.set_block_at::<1>(vfx, [1])?;
                                                crit_index = record;
                                                ctx.set_i32_at(
                                                    vfx.wrapping_add(VfxSlot::FRAME),
                                                    0,
                                                )?;

                                                break;
                                            }
                                        }
                                    }

                                    if get_savage_blow_vfx(ctx, faction, slot)? {
                                        let (x, y) = if slot != 0 {
                                            let x =
                                                ctx.i32_at(entity.wrapping_add(Entity::POS_X))?;
                                            let spread = call_rng(ctx, 0x2e);
                                            let y =
                                                ctx.i32_at(entity.wrapping_add(Entity::POS_Y))?;
                                            let rise = call_rng(ctx, 0x1f);

                                            (
                                                x.wrapping_sub(spread.wrapping_mul(10))
                                                    .wrapping_add(-0x1b5),
                                                y.wrapping_add(rise.wrapping_mul(10))
                                                    .wrapping_add(-0x52d),
                                            )
                                        } else {
                                            let x = ctx.i32_at(base.wrapping_add(Entity::POS_X))?;
                                            let spread = call_rng(ctx, 0x2e);
                                            let y = ctx.i32_at(base.wrapping_add(Entity::POS_Y))?;
                                            let rise = call_rng(ctx, 0x19);

                                            (
                                                x.wrapping_add(spread.wrapping_mul(10))
                                                    .wrapping_add(-0x3a9),
                                                y.wrapping_sub(rise.wrapping_mul(10))
                                                    .wrapping_add(-0x497),
                                            )
                                        };

                                        let mut sprite = EffectSprite::default();

                                        effect_sprite_init(&mut sprite, x, y);
                                        ctx.savage_vfx.push(sprite);
                                    }

                                    if get_toxic_vfx(ctx, faction, slot)? {
                                        let (x, y) = if slot != 0 {
                                            let x =
                                                ctx.i32_at(entity.wrapping_add(Entity::POS_X))?;
                                            let spread = call_rng(ctx, 0x2e);
                                            let y =
                                                ctx.i32_at(entity.wrapping_add(Entity::POS_Y))?;
                                            let rise = call_rng(ctx, 0x1f);

                                            (
                                                x.wrapping_sub(spread.wrapping_mul(10))
                                                    .wrapping_add(-0x1b5),
                                                y.wrapping_add(rise.wrapping_mul(10))
                                                    .wrapping_add(-0x52d),
                                            )
                                        } else {
                                            let x = ctx.i32_at(base.wrapping_add(Entity::POS_X))?;
                                            let spread = call_rng(ctx, 0x2e);
                                            let y = ctx.i32_at(base.wrapping_add(Entity::POS_Y))?;
                                            let rise = call_rng(ctx, 0x19);

                                            (
                                                x.wrapping_add(spread.wrapping_mul(10))
                                                    .wrapping_add(-0x3a9),
                                                y.wrapping_sub(rise.wrapping_mul(10))
                                                    .wrapping_add(-0x497),
                                            )
                                        };

                                        let mut sprite = EffectSprite::default();

                                        effect_sprite_init(&mut sprite, x, y);
                                        ctx.toxic_vfx.push(sprite);
                                    }

                                    if get_metal_killer_vfx(ctx, faction, slot)? {
                                        let (x, y) = if slot != 0 {
                                            let x =
                                                ctx.i32_at(entity.wrapping_add(Entity::POS_X))?;
                                            let spread = call_rng(ctx, 0x2e);
                                            let y =
                                                ctx.i32_at(entity.wrapping_add(Entity::POS_Y))?;
                                            let rise = call_rng(ctx, 0x1f);

                                            (
                                                x.wrapping_sub(spread.wrapping_mul(10))
                                                    .wrapping_add(-0x1b5),
                                                y.wrapping_add(rise.wrapping_mul(10))
                                                    .wrapping_add(-0x52d),
                                            )
                                        } else {
                                            let x = ctx.i32_at(base.wrapping_add(Entity::POS_X))?;
                                            let spread = call_rng(ctx, 0x2e);
                                            let y = ctx.i32_at(base.wrapping_add(Entity::POS_Y))?;
                                            let rise = call_rng(ctx, 0x19);

                                            (
                                                x.wrapping_add(spread.wrapping_mul(10))
                                                    .wrapping_add(-0x3a9),
                                                y.wrapping_sub(rise.wrapping_mul(10))
                                                    .wrapping_add(-0x497),
                                            )
                                        };

                                        let mut sprite = EffectSprite::default();

                                        effect_sprite_init(&mut sprite, x, y);
                                        ctx.metal_killer_vfx.push(sprite);
                                    }
                                }

                                let debris = debris_ring.wrapping_add(
                                    (slot as i64 as usize).wrapping_mul(AppContext::DEBRIS_STRIDE),
                                );

                                ctx.set_i32_at(debris.wrapping_add(Debris::TIMER), 0xc)?;

                                if slot != 0 {
                                    hit_sound = if hit_sound == 0 { 1 } else { hit_sound };

                                    if !first && get_entity_base_idx(ctx)? as u32 as i32 == slot {
                                        hit_sound = 2;
                                        set_hit_flash_timer(ctx, 1, slot, 4)?;
                                    }

                                    if get_crit_vfx(ctx, faction, slot)? {
                                        let vfx = AppContext::CRIT_VFX
                                            .wrapping_add(crit_index * AppContext::CRIT_VFX_STRIDE);
                                        let x = ctx.i32_at(entity.wrapping_add(Entity::POS_X))?;
                                        let spread = call_rng(ctx, 0x2e);

                                        ctx.set_i32_at(
                                            vfx.wrapping_add(VfxSlot::POS_X),
                                            x.wrapping_sub(spread.wrapping_mul(10))
                                                .wrapping_add(-0x1b5),
                                        )?;

                                        let y = ctx.i32_at(entity.wrapping_add(Entity::POS_Y))?;
                                        let rise = call_rng(ctx, 0x1f);

                                        ctx.set_i32_at(
                                            vfx.wrapping_add(VfxSlot::POS_Y),
                                            y.wrapping_add(rise.wrapping_mul(10))
                                                .wrapping_add(-0x52d),
                                        )?;
                                    }

                                    let x = ctx.i32_at(entity.wrapping_add(Entity::POS_X))?;
                                    let spread = call_rng(ctx, 0x2e);

                                    ctx.set_i32_at(
                                        debris.wrapping_add(Debris::POS_X),
                                        x.wrapping_sub(spread.wrapping_mul(10))
                                            .wrapping_add(-0x1b5),
                                    )?;

                                    let y = ctx.i32_at(entity.wrapping_add(Entity::POS_Y))?;
                                    let rise = call_rng(ctx, 0x1f);

                                    ctx.set_i32_at(
                                        debris.wrapping_add(Debris::POS_Y),
                                        y.wrapping_add(rise.wrapping_mul(10)).wrapping_add(-0x52d),
                                    )?;
                                    ctx.set_i32_at(
                                        debris.wrapping_add(Debris::VARIANT),
                                        ctx.i32_at(entity.wrapping_add(Entity::HIT_SPARK_TYPE))?,
                                    )?;
                                } else {
                                    if get_crit_vfx(ctx, faction, 0)? {
                                        let vfx = AppContext::CRIT_VFX
                                            .wrapping_add(crit_index * AppContext::CRIT_VFX_STRIDE);
                                        let x = ctx.i32_at(base.wrapping_add(Entity::POS_X))?;
                                        let spread = call_rng(ctx, 0x2e);

                                        ctx.set_i32_at(
                                            vfx.wrapping_add(VfxSlot::POS_X),
                                            x.wrapping_add(spread.wrapping_mul(10))
                                                .wrapping_add(-0x3a9),
                                        )?;

                                        let y = ctx.i32_at(base.wrapping_add(Entity::POS_Y))?;
                                        let rise = call_rng(ctx, 0x19);

                                        ctx.set_i32_at(
                                            vfx.wrapping_add(VfxSlot::POS_Y),
                                            y.wrapping_sub(rise.wrapping_mul(10))
                                                .wrapping_add(-0x497),
                                        )?;
                                    }

                                    let x = ctx.i32_at(base.wrapping_add(Entity::POS_X))?;
                                    let spread = call_rng(ctx, 0x2e);

                                    ctx.set_i32_at(
                                        debris.wrapping_add(Debris::POS_X),
                                        x.wrapping_add(spread.wrapping_mul(10))
                                            .wrapping_add(-0x3a9),
                                    )?;

                                    let y = ctx.i32_at(base.wrapping_add(Entity::POS_Y))?;
                                    let rise = call_rng(ctx, 0x19);

                                    ctx.set_i32_at(
                                        debris.wrapping_add(Debris::POS_Y),
                                        y.wrapping_sub(rise.wrapping_mul(10)).wrapping_add(-0x497),
                                    )?;

                                    let variant =
                                        ctx.i32_at(base.wrapping_add(Entity::HIT_SPARK_TYPE))? == 1;

                                    ctx.set_i32_at(
                                        debris.wrapping_add(Debris::VARIANT),
                                        variant as i32,
                                    )?;

                                    if ctx.i32_at(base.wrapping_add(Entity::STATE))? as u32 <= 1 {
                                        ctx.set_i32_at(base.wrapping_add(Entity::STATE), 1)?;
                                        ctx.set_i32_at(base.wrapping_add(Entity::FRAME), 4)?;
                                    }

                                    hit_sound = 2;

                                    if first {
                                        base_shake_start(ctx, 0);
                                    }
                                }

                                let hp_before = get_hp(ctx, faction, slot)?;

                                if !get_trait_dojo(ctx, faction, slot)? {
                                    let damage = get_frame_damage(ctx, faction, slot)?;

                                    add_hp(ctx, faction, slot, damage.wrapping_neg())?;
                                }

                                if first && get_drain_pct(ctx, 0, slot)? > 0 {
                                    let button = get_entity_button(ctx, 0, slot)?;

                                    if get_deck_cooldown(ctx, wallet, button)? > 0 {
                                        let drained = get_drain_pct(ctx, 0, slot)?;

                                        if let Some(gauge) =
                                            red_gauge.get_mut(button as i64 as usize)
                                        {
                                            *gauge = gauge.wrapping_add(drained);
                                        }

                                        let (x, y) = if slot != 0 {
                                            let x = ctx.i32_at(AppContext::entity_field(
                                                0,
                                                slot,
                                                Entity::POS_X,
                                            ))?;
                                            let spread = call_rng(ctx, 0x2e);
                                            let y = ctx.i32_at(AppContext::entity_field(
                                                0,
                                                slot,
                                                Entity::POS_Y,
                                            ))?;
                                            let rise = call_rng(ctx, 0x1f);

                                            (
                                                x.wrapping_sub(spread.wrapping_mul(10))
                                                    .wrapping_add(-0x1b5),
                                                y.wrapping_add(rise.wrapping_mul(10))
                                                    .wrapping_add(-0x52d),
                                            )
                                        } else {
                                            let x = ctx.i32_at(base.wrapping_add(Entity::POS_X))?;
                                            let spread = call_rng(ctx, 0x2e);
                                            let y = ctx.i32_at(base.wrapping_add(Entity::POS_Y))?;
                                            let rise = call_rng(ctx, 0x19);

                                            (
                                                x.wrapping_add(spread.wrapping_mul(10))
                                                    .wrapping_add(-0x3a9),
                                                y.wrapping_sub(rise.wrapping_mul(10))
                                                    .wrapping_add(-0x497),
                                            )
                                        };

                                        let mut sprite = EffectSprite::default();

                                        effect_sprite_init(&mut sprite, x, y);
                                        ctx.drain_vfx.push(sprite);
                                    }
                                }

                                if !(slot != 0
                                    || first
                                    || ctx.u8_at(AppContext::BASE_KILL_BLOCKED)? == 0)
                                    && get_hp(ctx, 1, 0)? <= 0
                                {
                                    set_hp(ctx, 1, 0, 1)?;
                                }

                                let survive =
                                    ctx.i32_at(entity.wrapping_add(Entity::SURVIVE_CHANCE))?;

                                if survive > 0 {
                                    let roll = call_rng(ctx, 0x64);

                                    if ctx.i32_at(entity.wrapping_add(Entity::SURVIVE_CHANCE))?
                                        >= roll
                                        && ctx.i32_at(entity.wrapping_add(Entity::HP))? <= 0
                                        && ctx.i32_at(entity.wrapping_add(Entity::SURVIVE_USED))?
                                            == 0
                                    {
                                        ctx.set_i32_at(entity.wrapping_add(Entity::HP), 1)?;
                                        ctx.set_i32_at(
                                            entity.wrapping_add(Entity::SURVIVE_USED),
                                            1,
                                        )?;
                                        turn_on_proc_badge(ctx, faction, slot, 3)?;
                                        play_sound(sound_manager(ctx)?, 0x32, None);
                                    }
                                }

                                'knock: {
                                    if ctx.i32_at(entity.wrapping_add(Entity::HP))? <= 0 {
                                        ctx.set_i32_at(entity.wrapping_add(Entity::HP), 0)?;

                                        if first {
                                            ctx.set_i32_at(
                                                AppContext::KILLS_SINCE_SPAWN_TICK,
                                                ctx.i32_at(AppContext::KILLS_SINCE_SPAWN_TICK)?
                                                    .wrapping_add(1),
                                            )?;
                                        } else {
                                            let unit_id = get_slot_unit_id(ctx, 1, slot)?;

                                            if ctx.enemy_kill_counts.contains_key(&unit_id) {
                                                let unit_id = get_slot_unit_id(ctx, 1, slot)?;
                                                let count = ctx
                                                    .enemy_kill_counts
                                                    .entry(unit_id)
                                                    .or_insert(0);

                                                *count = count.wrapping_add(1);
                                            } else {
                                                let unit_id = get_slot_unit_id(ctx, 1, slot)?;

                                                *ctx.enemy_kill_counts
                                                    .entry(unit_id)
                                                    .or_insert(0) = 1;
                                            }
                                        }

                                        if slot == 0 {
                                            break 'knock;
                                        }
                                    } else if slot == 0 {
                                        break 'knock;
                                    }

                                    if get_frame_damage(ctx, faction, slot)? == 0 {
                                        break 'knock;
                                    }

                                    if !first
                                        && has_castle_enemy(ctx)?
                                        && get_entity_base_idx(ctx)? as u32 as i32 == slot
                                    {
                                        break 'knock;
                                    }

                                    let max = get_max_hp(ctx, faction, slot)?;
                                    let knockbacks = get_knockbacks(ctx, faction, slot)?;
                                    let thresholds =
                                        ctx.i32_at(entity.wrapping_add(Entity::KNOCKBACKS))?;
                                    let mut crossed_now = 0i32;
                                    let mut crossed_before = 0i32;

                                    if thresholds > 0 {
                                        let step = max as f32 / knockbacks as f32;
                                        let hp =
                                            ctx.i32_at(entity.wrapping_add(Entity::HP))? as f32;

                                        crossed_now = thresholds;

                                        for mark in 0..thresholds {
                                            if mark as f32 * step >= hp {
                                                crossed_now = mark;

                                                break;
                                            }
                                        }

                                        let before = hp_before as f32;

                                        crossed_before = thresholds;

                                        for mark in 0..thresholds {
                                            if mark as f32 * step >= before {
                                                crossed_before = mark;

                                                break;
                                            }
                                        }
                                    }

                                    if get_entity_state(ctx, faction, slot)? == 4
                                        || get_entity_state(ctx, faction, slot)? == 0x15
                                    {
                                        break 'knock;
                                    }

                                    let knocked = (crossed_now != crossed_before
                                        && crossed_now
                                            != ctx
                                                .i32_at(entity.wrapping_add(Entity::KNOCKBACKS))?)
                                        || ctx.i32_at(entity.wrapping_add(Entity::SURVIVE_USED))?
                                            == 1;

                                    if knocked {
                                        set_entity_state(ctx, faction, slot, 3)?;
                                        set_entity_frame(ctx, faction, slot, 0)?;

                                        if get_zkill_hit(ctx, faction, slot)?
                                            && get_hp(ctx, faction, slot)? == 0
                                        {
                                            set_no_revive(ctx, faction, slot, 1)?;

                                            let mut free = None;

                                            for record in 0..0x1eusize {
                                                if ctx.u8_at(AppContext::ZKILL_VFX.wrapping_add(
                                                    record * AppContext::ZKILL_VFX_STRIDE,
                                                ))? == 0
                                                {
                                                    free = Some(record);

                                                    break;
                                                }
                                            }

                                            if let Some(record) = free {
                                                let vfx = AppContext::ZKILL_VFX.wrapping_add(
                                                    record * AppContext::ZKILL_VFX_STRIDE,
                                                );

                                                clear_zkill_vfx_slot(ctx, vfx)?;
                                                ctx.set_block_at::<1>(vfx, [1])?;

                                                let x =
                                                    ctx.i32_at(entity.wrapping_add(Entity::POS_X))?;
                                                let spread = call_rng(ctx, 0x2e);

                                                ctx.set_i32_at(
                                                    vfx.wrapping_add(VfxSlot::POS_X),
                                                    x.wrapping_sub(spread.wrapping_mul(10))
                                                        .wrapping_add(0xfa),
                                                )?;

                                                let y =
                                                    ctx.i32_at(entity.wrapping_add(Entity::POS_Y))?;
                                                let rise = call_rng(ctx, 0x1f);

                                                ctx.set_i32_at(
                                                    vfx.wrapping_add(VfxSlot::POS_Y),
                                                    y.wrapping_add(rise.wrapping_mul(10))
                                                        .wrapping_add(-0x96),
                                                )?;
                                                play_sound_in_battle(ctx, 0x3b)?;
                                            }
                                        }

                                        if ctx.i32_at(
                                            entity.wrapping_add(Entity::DOUBLE_BOUNTY_STATE),
                                        )? == 1
                                        {
                                            ctx.set_i32_at(
                                                entity.wrapping_add(Entity::DOUBLE_BOUNTY_STATE),
                                                2,
                                            )?;
                                        }

                                        if ctx.i32_at(entity.wrapping_add(Entity::SURVIVE_USED))?
                                            == 1
                                        {
                                            ctx.set_i32_at(
                                                entity.wrapping_add(Entity::SURVIVE_USED),
                                                2,
                                            )?;
                                        }
                                    } else if get_warp_timer(ctx, faction, slot)? > 0
                                        && get_hp(ctx, faction, slot)? > 0
                                    {
                                        set_entity_state(ctx, faction, slot, 0x12)?;
                                        set_entity_frame(ctx, faction, slot, 0)?;
                                        play_sound_in_battle(ctx, 0x49)?;

                                        break 'knock;
                                    } else if get_cannon_blast_hit(ctx, faction, slot)? {
                                        set_entity_state(ctx, faction, slot, 5)?;
                                        set_entity_frame(ctx, faction, slot, 0)?;
                                    } else if get_kb_proc_hit(ctx, faction, slot)? {
                                        set_entity_state(ctx, faction, slot, 6)?;
                                        set_entity_frame(ctx, faction, slot, 0)?;
                                    }

                                    set_warp_timer(ctx, faction, slot, 0)?;
                                }

                                if first {
                                    continue;
                                }

                                if slot != 0 && get_entity_base_idx(ctx)? as u32 as i32 != slot {
                                    continue;
                                }

                                let music_row = AppContext::MAP_STAGE_ROWS.wrapping_add(
                                    (ctx.i32_at(AppContext::STAGE_MUSIC_ROW)? as i64 as usize)
                                        .wrapping_mul(AppContext::MAP_STAGE_ROW_STRIDE),
                                );
                                let key = ctx.block_at::<4>(music_row.wrapping_add(0xb8))?;
                                let cell = ctx.block_at::<4>(music_row.wrapping_add(0xc))?;
                                let switch_percent = i32::from_le_bytes([
                                    cell[0] ^ key[0],
                                    cell[1] ^ key[1],
                                    cell[2] ^ key[2],
                                    cell[3] ^ key[3],
                                ]);

                                if switch_percent.wrapping_sub(1) as u32 > 0x62 {
                                    continue;
                                }

                                let max = get_max_hp(ctx, 1, slot)?;
                                let key = ctx.block_at::<4>(music_row.wrapping_add(0xb8))?;
                                let cell = ctx.block_at::<4>(music_row.wrapping_add(0xc))?;
                                let percent = i32::from_le_bytes([
                                    cell[0] ^ key[0],
                                    cell[1] ^ key[1],
                                    cell[2] ^ key[2],
                                    cell[3] ^ key[3],
                                ]);
                                let threshold = ops::div_100(percent.wrapping_mul(max));

                                if hp_before <= threshold {
                                    continue;
                                }

                                if get_hp(ctx, 1, slot)? > threshold {
                                    continue;
                                }

                                ctx.set_i32_at(AppContext::BGM_SWITCH_STATE, 1)?;
                                ctx.set_i32_at(AppContext::BGM_SWITCH_FRAME, 0)?;
                                ctx.set_i32_at(AppContext::BGM_SWITCH_FRAMES, 0x2d)?;

                                let key = ctx.block_at::<4>(music_row.wrapping_add(0xb8))?;
                                let cell = ctx.block_at::<4>(music_row.wrapping_add(0x10))?;

                                ctx.set_block_at::<4>(
                                    AppContext::BGM_SWITCH_NEXT,
                                    [
                                        cell[0] ^ key[0],
                                        cell[1] ^ key[1],
                                        cell[2] ^ key[2],
                                        cell[3] ^ key[3],
                                    ],
                                )?;
                                ctx.set_i32_at(AppContext::BGM_BOSS_PHASE, 1)?;
                                sound_manager(ctx)?.stop_audio(-1);
                            }

                            for button in 0..10i32 {
                                let drained = red_gauge[button as usize];

                                if drained == 0 {
                                    continue;
                                }

                                let recharge = get_unit_recharge(ctx, faction, button)?;
                                let cooldown = get_deck_cooldown(ctx, wallet, button)?;
                                let flash = wallet
                                    .wrapping_add(AppContext::WALLET_RED_GAUGE_FRAMES)
                                    .wrapping_add(button as usize * 4);

                                if ctx.i32_at(flash)? == 0
                                    || get_deck_cooldown_max(ctx, wallet, button)?
                                        > get_deck_cooldown(ctx, wallet, button)?
                                {
                                    let current = get_deck_cooldown(ctx, wallet, button)?;

                                    set_deck_cooldown_max(ctx, wallet, button, current)?;
                                }

                                let span = recharge.wrapping_sub(cooldown);
                                let refund = max_i32(
                                    ops::div_neg_100(drained.wrapping_mul(span))
                                        .wrapping_add(span),
                                    0,
                                );

                                set_deck_cooldown(
                                    ctx,
                                    wallet,
                                    button,
                                    recharge.wrapping_sub(refund),
                                    0,
                                )?;

                                let frames =
                                    get_setting(&ctx.settings, b"battle_red_gauge_frame", 0xf)?;

                                ctx.set_i32_at(flash, frames)?;
                            }
                        }

                        if get_battle_status(ctx)? == 0 {
                            if ctx.i32_at(AppContext::entity_field(1, 0, Entity::HP))? == 0 {
                                ctx.set_block_at::<8>(AppContext::OUTRO_PHASE, [0; 8])?;
                                ctx.set_block_at::<16>(
                                    AppContext::BGM_SWITCH_STATE,
                                    BGM_SWITCH_RESET,
                                )?;

                                if get_hp(ctx, 0, 0)? == 0 {
                                    set_hp(ctx, 0, 0, 1)?;
                                }

                                battle_init_win(ctx, 0)?;
                                battle_create_button(ctx)?;
                            } else if ctx.i32_at(AppContext::entity_field(0, 0, Entity::HP))? == 0 {
                                ctx.set_block_at::<8>(AppContext::OUTRO_PHASE, [0; 8])?;
                                ctx.set_block_at::<16>(
                                    AppContext::BGM_SWITCH_STATE,
                                    BGM_SWITCH_RESET,
                                )?;
                                on_battle_lost(ctx)?;
                                battle_create_button(ctx)?;
                            }
                        }

                        if get_battle_status(ctx)? == 0 {
                            if hit_sound == 2 {
                                play_sound(sound_manager(ctx)?, 0x16, None);
                            } else if hit_sound == 1 {
                                let variant = call_rng(ctx, 2).wrapping_add(0x14);

                                play_sound(sound_manager(ctx)?, variant, None);
                            }

                            for faction in 0..2i32 {
                                let first = faction == 0;
                                let wallet = AppContext::faction_flags(faction);

                                for button in 0..10i32 {
                                    let slot = button as usize * 4;
                                    let mut check_orb = first;

                                    if get_deck_cooldown(ctx, wallet, button)? > 0 {
                                        if get_unit_recharge(ctx, faction, button)?
                                            < get_deck_cooldown(ctx, wallet, button)?
                                        {
                                            let recharge = get_unit_recharge(ctx, faction, button)?;

                                            set_deck_cooldown(ctx, wallet, button, recharge, 1)?;
                                        } else {
                                            add_deck_cooldown(ctx, wallet, button, -1)?;
                                        }

                                        if first && get_deck_cooldown(ctx, wallet, button)? == 0 {
                                            play_sound(sound_manager(ctx)?, 0x1b, None);
                                        }
                                    }

                                    if check_orb {
                                        check_orb = get_button_unit_form(ctx, 0, button)? >= 2
                                            && slot_has_flagged_orb(ctx, 0, button)?
                                            && get_deck_cooldown(ctx, wallet, button)? == 0;
                                    }

                                    if check_orb {
                                        let deploys = ctx.i32_at(
                                            wallet
                                                .wrapping_add(AppContext::WALLET_DEPLOY_COUNTS)
                                                .wrapping_add(slot),
                                        )?;
                                        let seen = wallet
                                            .wrapping_add(AppContext::WALLET_ORB_DEPLOYS_SEEN)
                                            .wrapping_add(slot);

                                        if ctx.i32_at(seen)? <= deploys {
                                            ctx.set_i32_at(seen, deploys.wrapping_add(1))?;

                                            if orb_deploy_condition(ctx, wallet, 0, button)? {
                                                ctx.set_i32_at(
                                                    wallet
                                                        .wrapping_add(AppContext::WALLET_SLOT_FLASH)
                                                        .wrapping_add(slot),
                                                    0,
                                                )?;
                                            }
                                        }
                                    }

                                    if ctx.i32_at(
                                        wallet
                                            .wrapping_add(AppContext::WALLET_CONJURE_READY)
                                            .wrapping_add(slot),
                                    )? == 1
                                    {
                                        let lockout = wallet
                                            .wrapping_add(AppContext::WALLET_CONJURE_LOCKOUT)
                                            .wrapping_add(slot);
                                        let left = ctx.i32_at(lockout)?;

                                        if left > 0 {
                                            ctx.set_i32_at(lockout, left.wrapping_sub(1))?;
                                        }
                                    }

                                    let gauge = wallet
                                        .wrapping_add(AppContext::WALLET_RED_GAUGE_FRAMES)
                                        .wrapping_add(slot);
                                    let left = ctx.i32_at(gauge)?;

                                    if left > 0 {
                                        ctx.set_i32_at(gauge, left.wrapping_sub(1))?;
                                    }
                                }

                                let map_id = get_global_map_id(ctx, 0)?;

                                if !get_special_rule(ctx, &ctx.special_rules, map_id, 0)? {
                                    let income = get_money_increment(ctx, wallet)?;

                                    add_money(ctx, wallet, income)?;
                                }

                                if cannon_unlocked(ctx)? {
                                    let countdown = AppContext::entity_field(
                                        faction,
                                        0,
                                        Base::CANNON_COUNTDOWN,
                                    );
                                    let left = ctx.i32_at(countdown)?;

                                    if left > 0 {
                                        ctx.set_i32_at(countdown, left.wrapping_sub(1))?;

                                        if left.wrapping_sub(1) == 0 && first {
                                            play_sound(sound_manager(ctx)?, 0x1c, None);
                                        }
                                    }
                                }
                            }
                        }

                        let worker = ctx.i32_at(AppContext::WORKER_UPGRADE_VFX)?;

                        if worker > 0 {
                            ctx.set_i32_at(AppContext::WORKER_UPGRADE_VFX, worker.wrapping_sub(1))?;
                        }

                        for spark in 0..50usize {
                            let record = AppContext::PENDING_STRIKE_SPARKS
                                .wrapping_add(spark * AppContext::PENDING_STRIKE_SPARKS_STRIDE);
                            let first = ctx.i32_at(record)?;

                            if first > 0 {
                                ctx.set_i32_at(record, first.wrapping_sub(1))?;
                            }

                            let second = ctx.i32_at(record.wrapping_add(0xc))?;

                            if second > 0 {
                                ctx.set_i32_at(record.wrapping_add(0xc), second.wrapping_sub(1))?;
                            }
                        }

                        debris_tick(ctx)?;
                        crit_vfx_tick(ctx)?;
                        zkill_vfx_tick(ctx)?;
                        barrier_vfx_tick(ctx)?;
                        shield_vfx_tick(ctx)?;
                        savage_vfx_tick(ctx)?;
                        toxic_vfx_tick(ctx)?;
                        metal_killer_vfx_tick(ctx)?;
                        drain_vfx_tick(ctx)?;
                        base_guard_notice_tick(ctx)?;
                    }

                    step += 1;
                }

                background_particles(ctx)?;
                ctx.set_i32_at(
                    AppContext::DRAW_FRAMES,
                    ctx.i32_at(AppContext::DRAW_FRAMES)?.wrapping_add(1),
                )?;

                let blink = ctx.i32_at(AppContext::BLINK_COUNTER)?;

                ctx.set_i32_at(AppContext::BLINK_COUNTER, blink.wrapping_add(1))?;

                if blink.wrapping_add(1) == 3 {
                    ctx.set_i32_at(AppContext::BLINK_ON, 1)?;
                } else if blink >= 5 {
                    ctx.set_i32_at(AppContext::BLINK_COUNTER, 0)?;
                    ctx.set_i32_at(AppContext::BLINK_ON, 0)?;
                }

                match ctx.i32_at(AppContext::BGM_SWITCH_STATE)? {
                    0 => {
                        ctx.set_block_at::<8>(AppContext::BGM_SWITCH_STATE, [0; 8])?;
                        ctx.set_i32_at(AppContext::BGM_SWITCH_FRAMES, 0)?;
                        ctx.set_i32_at(AppContext::BGM_SWITCH_NEXT, -1)?;
                    }
                    1 => {
                        let frame = ctx.i32_at(AppContext::BGM_SWITCH_FRAME)?.wrapping_add(1);

                        ctx.set_i32_at(AppContext::BGM_SWITCH_FRAME, frame)?;

                        if frame == ctx.i32_at(AppContext::BGM_SWITCH_FRAMES)? {
                            sound_manager(ctx)?.stop_audio(-1);

                            if ctx.i32_at(AppContext::BGM_SWITCH_NEXT)? != -1 {
                                bgm_player_switch(ctx, 0, 1)?;
                            }

                            ctx.set_block_at::<8>(AppContext::BGM_SWITCH_STATE, [0; 8])?;
                            ctx.set_i32_at(AppContext::BGM_SWITCH_FRAMES, 0)?;
                            ctx.set_i32_at(AppContext::BGM_SWITCH_NEXT, -1)?;
                        }
                    }
                    _ => {}
                }

                ctx.set_f32_at(
                    AppContext::CAT_GOD_SPIN,
                    ctx.f32_at(AppContext::CAT_GOD_SPIN)? + 0.5,
                )?;
                ctx.set_f32_at(
                    AppContext::CAT_GOD_GLOW,
                    0.5 + ctx.f32_at(AppContext::CAT_GOD_GLOW)?,
                )?;
                break 'frame;
            }
        }

        if ctx.u8_at(AppContext::CAT_GOD_MENU_IS_OPEN)? != 0 && !cat_god_menu_input(ctx)? {
            return Ok(false);
        }
    }

    ctx.set_i32_at(
        AppContext::BATTLE_TICKS,
        ctx.i32_at(AppContext::BATTLE_TICKS)?.wrapping_add(1),
    )?;

    if get_battle_status(ctx)? == 0 {
        bgm_player_tick(ctx)?;
    }

    let style = ctx.i32_at(AppContext::CURTAIN_STYLE)?;

    if !fade_update(ctx, style)? {
        return Ok(false);
    }

    let lockout = ctx.i32_at(AppContext::UI_TAP_LOCKOUT)?;

    ctx.set_i32_at(
        AppContext::UI_TAP_LOCKOUT,
        if lockout < 2 {
            0
        } else {
            lockout.wrapping_sub(1)
        },
    )?;

    Ok(true)
}
