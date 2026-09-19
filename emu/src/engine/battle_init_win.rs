use std::rc::Rc;

use crate::{operation, Fault};

use super::{
    ad_prepare, add_resource, add_stage_record, add_stage_unlock, add_stages_cleared, ads_available, aku_timer_set, altar_stage_value,
    analytics_named, analytics_params, battle_init_win_lambda_0, breadcrumb, breadcrumb_with, calculate_treasure_percentages, call_rng, check_medals,
    clear_count_rewards_get, clear_ex_replacement_stage, clear_lineup_count, clear_lineup_record, commit_stage_score, compute_stage_xp,
    config_json_int, dialog_origin, dialog_top, enigma_active_at, enigma_active_count, enigma_add_stamina, enigma_group_at, enigma_medal_count,
    enigma_prune, enigma_roll, event_reward_received, event_reward_set, event_unit_slot, ex_redirect_check_a, feature_enabled, find_item_by_kind,
    find_item_index, get_aku_stage_list, get_cleared_count, get_drawable_width, get_entity_base_idx, get_global_map_id, get_item_count,
    get_map_count, get_map_index, get_map_type, get_point_cap, get_point_id, get_point_rewards, get_point_total, get_powerup, get_release_point_cap,
    get_slot_unit_id, get_stage_best_score, get_stage_count, get_stage_index, get_stage_record, get_stage_score, get_stage_set_size,
    get_stages_cleared, get_star_level, get_trait_dojo, grant_stage_reward, has_point_decay, invasion_available, invasion_z_available,
    is_aku_final_map, is_ex_option_target, is_reward_claimed, is_score_stage, labyrinth_active, labyrinth_result_ready, labyrinth_roll_floor,
    labyrinth_submit, labyrinth_unit_count, log_analytics_event, map_guerrilla_set, map_index_of_map_id, map_interval, map_one_time,
    map_type_base_id, map_type_of_map_id, map_xp_ad, max_i32, mission_mark, mission_progress, mission_progress_list, new_button_register,
    new_button_set_touchable, notification_schedule, now_seconds, obf_value_add, obf_value_read, obf_value_set, orb_def_count, orb_inventory_count,
    play_sound, point_reward_analytics, record_stage_lineup, record_stage_played, replay_mode, request_save_data, reward_ad_ready, reward_owned,
    reward_unit_id, roll_drop_item_counts, roll_filibuster_stage, server_config_int, set_auto_camera_mode, set_battle_status, set_entity_state,
    set_fever_fade_out, set_map_open, set_reward_claimed, set_stage_record, set_stage_unlock, shop_offer_start, sound_manager, stage_pair_progress,
    stage_pair_progress_set, stage_pair_record, stage_reward_item, stage_reward_kind, stage_reward_taken, stage_stamina_cost, string_format_int,
    treasure_area_match, treasure_festival_active, treasure_group_at, treasure_group_cleared, treasure_group_of_stage, ui_node_add_child,
    ui_node_set_anchor, ui_node_set_panel, ui_node_set_sprite, ui_node_set_zoom, unit_buy_field, validate_map_type, vibration_reset,
    web_popup_request, web_popup_stage_match, xor_row46_get, AppContext, ENTITY_BASE, ENTITY_STRIDE, Entity, FACTION_STRIDE, FormatArg, UNIT_BUY,
    UNIT_BUY_STRIDE,
};

const SITE: &str = "battle_init_win";

pub fn battle_init_win(ctx: &mut AppContext, cleared: u8) -> Result<(), Fault> {
    let map_id = get_global_map_id(ctx, 0)?;
    let map_text = string_format_int(ctx, b"%d", map_id)?;
    let star = get_star_level(ctx)?;
    let star_text = string_format_int(ctx, b"%d", star)?;
    let stage = get_stage_index(ctx)?;
    let stage_text = string_format_int(ctx, b"%d", stage)?;

    breadcrumb_with(ctx, 0x47, &[(b"MapID", &map_text), (b"Level", &star_text), (b"StageIndex", &stage_text)])?;
    ctx.set_block_at::<2>(AppContext::RESULT_VIDEO_BUTTON, [0; 2])?;
    ctx.item_snapshot.clear();

    for item in 0..0x113 {
        let count = get_item_count(ctx, item)?;

        *ctx.item_snapshot.entry(item).or_insert(0) = count;
    }

    let mut orb = 0i32;

    while (orb as i64) < orb_def_count(&ctx.orb_store) {
        let count = orb_inventory_count(ctx, orb);

        *ctx.item_snapshot.entry(orb.wrapping_add(0x7530)).or_insert(0) = count;
        orb += 1;
    }

    ctx.set_block_at::<8>(AppContext::LOST_MAP_TYPE, [0xff; 8])?;
    ctx.set_i32_at(AppContext::LOST_STAGE, -1)?;

    if cleared == 0 {
        set_auto_camera_mode(ctx, 2)?;
        sound_manager(ctx)?.stop_audio(-1);
        vibration_reset(ctx)?;
    }

    'voice: {
        let map_id = get_global_map_id(ctx, 0)?;

        if !ctx.map_records.contains_key(&map_id) {
            play_sound(sound_manager(ctx)?, 8, None);

            break 'voice;
        }

        let map_id = get_global_map_id(ctx, 0)?;

        if ctx.map_records.entry(map_id).or_default().victory_voices.is_empty() {
            play_sound(sound_manager(ctx)?, 8, None);

            break 'voice;
        }

        let map_id = get_global_map_id(ctx, 0)?;

        if ctx.map_records.entry(map_id).or_default().victory_voice_mode != 0 {
            let map_id = get_global_map_id(ctx, 0)?;

            if ctx.map_records.entry(map_id).or_default().victory_voice_mode != 1 {
                break 'voice;
            }

            let map_id = get_global_map_id(ctx, 0)?;
            let mut index = 0usize;

            while index < ctx.map_records.entry(map_id).or_default().victory_voices.len() {
                let voice = *ctx.map_records.entry(map_id).or_default().victory_voices.get(index).ok_or(Fault::IndexOutOfRange { site: SITE, index: index as i64, limit: 0 })?;

                if voice != -1 {
                    play_sound(sound_manager(ctx)?, voice, None);
                }

                index += 1;
            }

            break 'voice;
        }

        let first = get_global_map_id(ctx, 0)?;
        let second = get_global_map_id(ctx, 0)?;
        let count = ctx.map_records.entry(second).or_default().victory_voices.len() as u32;
        let pick = call_rng(ctx, count as i32) as i64;
        let voices = &ctx.map_records.entry(first).or_default().victory_voices;
        let voice = *voices.get(pick as usize).ok_or(Fault::OutOfRange { site: SITE })?;

        if voice != -1 {
            play_sound(sound_manager(ctx)?, voice, None);
        }
    }

    for slot in 1..0x33i32 {
        let entity = ENTITY_BASE + FACTION_STRIDE + slot as usize * ENTITY_STRIDE;

        if ctx.i32_at(entity + Entity::OCCUPANT)? != 0 && ctx.i32_at(entity + Entity::STATE)? != 4 && slot != get_entity_base_idx(ctx)? {
            ctx.set_i32_at(entity + Entity::STATE, 4)?;
            ctx.set_i32_at(entity + Entity::FRAME, 0)?;
            ctx.set_i32_at(entity + Entity::HP, 0)?;

            let unit_id = get_slot_unit_id(ctx, 1, slot)?;

            if ctx.enemy_kill_counts.contains_key(&unit_id) {
                let unit_id = get_slot_unit_id(ctx, 1, slot)?;
                let count = ctx.enemy_kill_counts.entry(unit_id).or_insert(0);

                *count = count.wrapping_add(1);
            } else {
                let unit_id = get_slot_unit_id(ctx, 1, slot)?;

                *ctx.enemy_kill_counts.entry(unit_id).or_insert(0) = 1;
            }
        }

        if slot == get_entity_base_idx(ctx)? && !get_trait_dojo(ctx, 1, slot)? {
            set_entity_state(ctx, 1, slot, 0x11)?;
        }
    }

    let kills: Vec<(i32, i32)> = ctx.enemy_kill_counts.iter().map(|(unit, count)| (*unit, *count)).collect();

    for (unit, count) in kills {
        let stage = get_stage_index(ctx)?;
        let star = if ctx.i32_at(AppContext::CHAPTER_MODE)? == 3 { get_star_level(ctx)? } else { 0 };
        let map_id = get_global_map_id(ctx, 0)?;

        mission_progress_list(ctx, 0x16, &[stage, star, unit, map_id], count)?;
    }

    let replay = replay_mode(ctx)?;

    if !replay && cleared == 0 {
        record_stage_lineup(ctx)?;
    }

    if labyrinth_active(ctx)? {
        labyrinth_roll_floor(ctx, 0)?;
    }

    ctx.set_block_at::<1>(AppContext::POINT_LIMIT_PENDING, [0])?;

    if is_score_stage(ctx.event_items.as_ref()) {
        let store = ctx.event_items.as_ref().ok_or(Fault::NullPointer { site: SITE })?;
        let total = get_point_total(store);

        ctx.set_block_at::<1>(AppContext::POINT_LIMIT_PENDING, [(get_stage_score(store).wrapping_add(total) > get_point_cap(store)) as u8])?;

        let score = get_stage_score(ctx.event_items.as_ref().ok_or(Fault::NullPointer { site: SITE })?);
        let stage = get_stage_index(ctx)?;
        let best = get_stage_best_score(ctx.event_items.as_ref().ok_or(Fault::NullPointer { site: SITE })?, stage);

        ctx.set_i32_at(AppContext::NEW_BEST_SCORE, (score > best) as i32)?;
        commit_stage_score(ctx)?;
        set_fever_fade_out(&mut ctx.special_rules);
    }

    set_battle_status(ctx, 1)?;

    let xp = compute_stage_xp(ctx)?;

    ctx.set_i32_at(AppContext::WIN_XP, xp)?;

    let mut xp = ctx.i32_at(AppContext::WIN_XP)?;

    if get_global_map_id(ctx, 0)? == 0xbb8 {
        xp = operation::div_100(server_config_int(ctx, b"CnfGetXpUp", 0x64, 0x1f4)?.wrapping_mul(xp));
        ctx.set_i32_at(AppContext::WIN_XP, xp)?;
    }

    let mut cell = ctx.block_at::<8>(AppContext::ITEM_6_COUNT)?;

    obf_value_add(&mut cell, xp);
    ctx.set_block_at::<8>(AppContext::ITEM_6_COUNT, cell)?;

    if obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_6_COUNT)?) as i32 >= 0x5f5e0ff {
        let mut cell = ctx.block_at::<8>(AppContext::ITEM_6_COUNT)?;

        obf_value_set(&mut cell, 0x5f5e0ff);
        ctx.set_block_at::<8>(AppContext::ITEM_6_COUNT, cell)?;
    }

    ctx.set_block_at::<1>(AppContext::EX_OFFERED, [0])?;

    if cleared == 0 {
        let mut index = 0usize;

        while (index as i64) < ctx.combo_store.records.len() as i32 as i64 {
            let record = ctx.combo_store.records.get(index).ok_or(Fault::IndexOutOfRange { site: SITE, index: index as i64, limit: 0 })?;
            let (enabled, kind) = (record.enabled, record.kind[0]);

            if enabled != 0 {
                mission_mark(ctx, 0xf, kind)?;
                mission_progress(ctx, 0xf, kind, 1, 0, 0)?;
            }

            index += 1;
        }
    }

    let map_key = map_type_base_id(validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?), ctx.i32_at(AppContext::MAP_INDEX)?);

    if ctx.i32_at(AppContext::CHAPTER_MODE)? != 0x63 && map_one_time(ctx, map_key) {
        let count = ctx.map_clear_counts.entry(map_key).or_insert(0);

        *count = count.wrapping_add(1);

        if map_interval(ctx, map_key) != 0 && !ctx.map_reopen_times.contains_key(&map_key) {
            let now = now_seconds(ctx)?;
            let minutes = map_interval(ctx, map_key);

            *ctx.map_reopen_times.entry(map_key).or_insert(0.0) = now + minutes.wrapping_mul(0x3c) as f64;
        }
    }

    if ctx.i32_at(AppContext::FIRST_WIN_GATE_A)? | ctx.i32_at(AppContext::FIRST_WIN_GATE_B)? == 0 {
        ctx.set_block_at::<1>(AppContext::FIRST_WIN_PENDING, [1])?;
    }

    ctx.set_block_at::<8>(AppContext::DROP_RATE, [0; 8])?;
    ctx.reward_queue.clear();

    if get_global_map_id(ctx, 0)? == 0xbb8 && get_stage_index(ctx)? == 6 && ctx.u8_at(AppContext::TUTORIAL_STAGE_SIX)? != 0 && get_stage_record(ctx, -2, 0, 6, 0, 0)? == 0 {
        ctx.set_block_at::<1>(AppContext::TUTORIAL_STAGE_SIX, [0])?;
    }

    let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
    let use_cache = replay as i32;
    let mut first_record = false;
    let pair_map;
    let pair_stage;

    if chapter == 3 {
        pair_stage = ctx.i32_at(AppContext::STAGE_ROW)?;
        pair_map = get_global_map_id(ctx, 0)?;
        log_analytics_event(ctx, 2, 0, 0, 0, 0)?;
        mission_progress(ctx, 0, pair_map, 1, cleared as i32, 0)?;
        mission_progress(ctx, 1, pair_map.wrapping_mul(0x64).wrapping_add(pair_stage), 1, cleared as i32, 0)?;
        mission_progress(ctx, 9, operation::div_1000(pair_map), 1, cleared as i32, 0)?;

        if get_map_type(ctx, 0)? == -0x17 {
            let star = get_star_level(ctx)?;

            mission_progress(ctx, 0x1e, star.wrapping_add(pair_stage.wrapping_mul(10)), 1, cleared as i32, 0)?;
        }

        let bits = map_guerrilla_set(ctx, pair_map);

        if bits > 0 {
            let mut bit = 0i32;
            let mut mask = 1i32;

            loop {
                if mask & bits != 0 {
                    mission_progress(ctx, 0x14, bit, 1, cleared as i32, 0)?;
                }

                mask = 2i32.wrapping_shl(bit as u32 & 0x1f);
                bit += 1;

                if mask > bits {
                    break;
                }
            }
        }

        ctx.set_i32_at(AppContext::RESULT_CHAPTER_MODE, ctx.i32_at(AppContext::CHAPTER_MODE)?)?;
        ctx.set_i32_at(AppContext::RESULT_ENTRY_STAGE, ctx.i32_at(AppContext::ENTRY_STAGE)?)?;
        ctx.set_i32_at(AppContext::RESULT_MAP_LOCKED, 0)?;
        ctx.set_i32_at(AppContext::NEXT_STAGE_UNLOCKED, -1)?;

        let map_type = validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?);
        let map_index = ctx.i32_at(AppContext::MAP_INDEX)?;
        let star = ctx.i32_at(AppContext::STAR_LEVEL)?;
        let target = if get_stages_cleared(ctx, map_type, map_index, star, use_cache)? != ctx.i32_at(AppContext::STAGE_ROW)? {
            AppContext::RESULT_STAGE_CLEARED
        } else {
            add_stages_cleared(ctx, map_type, map_index, star, 1, use_cache)?;

            let cleared_now = get_stages_cleared(ctx, map_type, map_index, star, use_cache)?;

            ctx.set_i32_at(AppContext::RESULT_STAGE_CLEARED, cleared_now)?;
            ctx.set_i32_at(AppContext::RESULT_NEW_CLEAR, 0)?;
            ctx.set_i32_at(AppContext::RESULT_NEW_CLEAR + 4, -1)?;
            add_stage_unlock(ctx, map_type, map_index, star, 1, use_cache)?;

            let cleared_now = get_stages_cleared(ctx, map_type, map_index, star, use_cache)?;
            let data_id = ctx.i32_at(AppContext::MAP_DATA_ID)?;
            let set = ctx.i32_at(AppContext::MAP_STAGE_SET)?;
            let size = get_stage_set_size(ctx.map_data.entry(data_id).or_default(), set)?;

            if cleared_now as i64 as u64 == size {
                let data_id = ctx.i32_at(AppContext::MAP_DATA_ID)?;
                let set = ctx.i32_at(AppContext::MAP_STAGE_SET)?;
                let size = get_stage_set_size(ctx.map_data.entry(data_id).or_default(), set)?;

                set_stage_unlock(ctx, map_type, map_index, star, (size as i32).wrapping_sub(1), use_cache)?;

                if ctx.i32_at(AppContext::STAR_LEVEL)? == 0 && validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?) == 0 {
                    let map_type = validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?);
                    let next = ctx.i32_at(AppContext::MAP_INDEX)?.wrapping_add(1);

                    set_map_open(ctx, map_type, next, 0, 1)?;
                }

                ctx.set_i32_at(AppContext::NEXT_STAGE_UNLOCKED, 0)?;
                add_resource(ctx, 0x16, 0x1e, 0)?;

                let star = ctx.i32_at(AppContext::STAR_LEVEL)?;

                mission_progress(ctx, 3, star, 1, cleared as i32, use_cache)?;
                add_resource(ctx, 0x69, 1, 0)?;

                let map_id = get_global_map_id(ctx, 0)?;
                let stage = get_stage_index(ctx)?;
                let star = get_star_level(ctx)?;

                analytics_params(
                    ctx,
                    0x98c17f,
                    0x1e,
                    &[
                        (b"sec1_type", FormatArg::Text(b"MapID")),
                        (b"sec1_id", FormatArg::Int(map_id)),
                        (b"sec2_type", FormatArg::Text(b"StageIdx")),
                        (b"sec2_id", FormatArg::Int(stage)),
                        (b"ex_type", FormatArg::Text(b"StageLv")),
                        (b"ex_id", FormatArg::Int(star)),
                    ],
                )?;
            }

            AppContext::UNIT_UNLOCK_NOTICE
        };

        ctx.set_i32_at(target, -1)?;

        let map_type = validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?);
        let map_index = ctx.i32_at(AppContext::MAP_INDEX)?;
        let stage_row = ctx.i32_at(AppContext::STAGE_ROW)?;
        let star = ctx.i32_at(AppContext::STAR_LEVEL)?;

        if !replay {
            let score = is_score_stage(ctx.event_items.as_ref());
            let decay = score && ctx.event_items.as_ref().is_some_and(has_point_decay);
            let mut record = 0;

            if !score || !decay {
                add_stage_record(ctx, map_type, map_index, stage_row, star, 1, 0)?;
                record = get_stage_record(ctx, map_type, map_index, stage_row, star, 0)?;
            }

            if !(score && decay) && record >= 0x2710 {
                set_stage_record(ctx, map_type, map_index, stage_row, star, 0x270f, 0)?;
            }
        } else {
            let before = get_stage_record(ctx, map_type, map_index, stage_row, star, 1)?;

            if before <= 0x270e {
                let lineups = clear_lineup_count(ctx, -1, 0, 0)?.wrapping_add(1);

                set_stage_record(ctx, map_type, map_index, stage_row, star, max_i32(lineups, before), 1)?;
            }

            first_record = get_stage_record(ctx, map_type, map_index, stage_row, star, 1)? > before;
        }

        let aku = get_aku_stage_list(ctx, 0);
        let record = get_stage_record(ctx, map_type, map_index, stage_row, star, use_cache)?;

        if record == 1 && validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?) == -0x13 && ctx.i32_at(AppContext::MAP_INDEX)? == 0 && ctx.i32_at(AppContext::STAR_LEVEL)? == 0 && aku.len() == 1 {
            let only = *aku.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?;

            if only == ctx.i32_at(AppContext::STAGE_ROW)? && (only == 0x30 || only == 0) {
                aku_timer_set(ctx, 0, 0.0);
            }
        }

        if validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?) == 0
            && ctx.i32_at(AppContext::MAP_INDEX)? == 0x30
            && ctx.i32_at(AppContext::STAGE_ROW)? == get_stage_count(ctx, 0, 0x30)?.wrapping_sub(1)
            && get_stage_record(ctx, map_type, map_index, stage_row, star, use_cache)? == 1
        {
            ctx.set_i32_at(AppContext::RESULT_MAP_LOCKED, 1)?;
            ctx.set_i32_at(AppContext::RESULT_NEW_CLEAR, 1)?;
        }

        if validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?) == -9 && ctx.i32_at(AppContext::MAP_INDEX)? == get_map_count(ctx, -9)?.wrapping_sub(1) {
            let last_map = get_map_count(ctx, -9)?.wrapping_sub(1);

            if ctx.i32_at(AppContext::STAGE_ROW)? == get_stage_count(ctx, -9, last_map)?.wrapping_sub(1) && get_stage_record(ctx, map_type, map_index, stage_row, star, use_cache)? == 1 {
                ctx.set_i32_at(AppContext::RESULT_MAP_LOCKED, 1)?;
            }
        }
    } else if chapter != 0x63 {
        pair_stage = ctx.i32_at(AppContext::STAGE_ROW)?;
        pair_map = get_global_map_id(ctx, 0)?;

        let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;

        ctx.set_i32_at(AppContext::RESULT_CHAPTER_MODE, chapter)?;
        ctx.set_i32_at(AppContext::RESULT_ENTRY_STAGE, ctx.i32_at(AppContext::ENTRY_STAGE)?)?;

        let stage_row = ctx.i32_at(AppContext::STAGE_ROW)?;
        let records = ctx.bytes_from((AppContext::STAGE_RECORD_CHAPTERS as i64 + (chapter as i64) * 0xd0) as usize)?;
        let record = operation::xor_row_decode(records, 0x33, stage_row as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: stage_row as i64, limit: 0x33 })?;

        if record == 0 {
            log_analytics_event(ctx, 1, chapter, stage_row, 0, 0)?;
        }

        if ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? == 0 && ctx.u8_at(AppContext::BATTLE_IS_INVASION)? == 0 && ctx.u8_at(AppContext::BATTLE_IS_Z_INVASION)? == 0 {
            let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
            let stage_row = ctx.i32_at(AppContext::STAGE_ROW)?;

            log_analytics_event(ctx, 0x21, chapter, stage_row, 0, 0)?;
        }

        mission_progress(ctx, 0, pair_map, 1, cleared as i32, 0)?;
        mission_progress(ctx, 1, pair_map.wrapping_mul(0x64).wrapping_add(pair_stage), 1, cleared as i32, 0)?;
        mission_progress(ctx, 9, operation::div_1000(pair_map), 1, cleared as i32, 0)?;
        ctx.set_i32_at(AppContext::RESULT_MAP_LOCKED, 0)?;

        let mut chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
        let progress_cell = (AppContext::CHAPTER_PROGRESS as i64 + (chapter as i64) * 4) as usize;
        let key = ctx.block_at::<4>(AppContext::CHAPTER_PROGRESS_KEY)?;
        let raw = ctx.block_at::<4>(progress_cell)?;
        let progress = u32::from_le_bytes([raw[0] ^ key[0], raw[1] ^ key[1], raw[2] ^ key[2], raw[3] ^ key[3]]);

        if progress as i32 != ctx.i32_at(AppContext::STAGE_ROW)? {
            ctx.set_i32_at(AppContext::RESULT_STAGE_CLEARED, -1)?;
        } else {
            let next = progress.wrapping_add(1).to_le_bytes();

            ctx.set_block_at::<4>(progress_cell, [key[0] ^ next[0], key[1] ^ next[1], key[2] ^ next[2], key[3] ^ next[3]])?;

            let chapter_now = ctx.i32_at(AppContext::CHAPTER_MODE)?;
            let progress_row = ctx.bytes_from(AppContext::CHAPTER_PROGRESS)?;
            let progress = operation::xor_row_decode(progress_row, 10, chapter_now as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: chapter_now as i64, limit: 10 })?;

            ctx.set_i32_at(AppContext::RESULT_STAGE_CLEARED, progress as i32)?;
            ctx.set_i32_at(AppContext::RESULT_NEW_CLEAR, 0)?;
            ctx.set_i32_at(AppContext::RESULT_NEW_CLEAR + 4, -1)?;

            let unlocks = ((chapter_now as i64) * 4 + AppContext::STAGE_UNLOCK_CHAPTERS as i64) as usize;

            ctx.set_i32_at(unlocks, ctx.i32_at(unlocks)?.wrapping_add(1))?;
            ctx.set_i32_at(AppContext::UNIT_UNLOCK_NOTICE, -1)?;

            chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;

            let progress_row = ctx.bytes_from(AppContext::CHAPTER_PROGRESS)?;
            let progress = operation::xor_row_decode(progress_row, 10, chapter as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: chapter as i64, limit: 10 })?;

            if progress == 0x30 {
                ctx.set_i32_at(AppContext::RESULT_MAP_LOCKED, 1)?;
                ctx.set_i32_at(AppContext::RESULT_NEW_CLEAR, 1)?;
                ctx.set_i32_at(((chapter as i64) * 4 + AppContext::STAGE_UNLOCK_CHAPTERS as i64) as usize, 0x2f)?;
                chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;

                let cleared_stage = ctx.i32_at(AppContext::RESULT_STAGE_CLEARED)?;

                for unit in 0..0x36cusize {
                    let row = ctx.bytes_from(UNIT_BUY + unit * UNIT_BUY_STRIDE)?;

                    if unit_buy_field(row, 0xf)? == chapter && unit_buy_field(row, 0)? == cleared_stage && unit_buy_field(row, 1)? == 0 {
                        ctx.set_i32_at(AppContext::RESULT_NEW_CLEAR + 4, unit as i32)?;
                    }
                }
            }
        }

        let stage_row = ctx.i32_at(AppContext::STAGE_ROW)?;
        let record_at = (AppContext::STAGE_RECORD_CHAPTERS as i64 + (chapter as i64) * 0xd0) as usize;
        let key = ctx.block_at::<4>(record_at + 0xcc)?;
        let raw = ctx.block_at::<4>(record_at + stage_row as i64 as usize * 4)?;
        let next = u32::from_le_bytes([raw[0] ^ key[0], raw[1] ^ key[1], raw[2] ^ key[2], raw[3] ^ key[3]]).wrapping_add(1).to_le_bytes();

        ctx.set_block_at::<4>(record_at + stage_row as i64 as usize * 4, [key[0] ^ next[0], key[1] ^ next[1], key[2] ^ next[2], key[3] ^ next[3]])?;

        let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
        let stage_row = ctx.i32_at(AppContext::STAGE_ROW)?;
        let record_at = (AppContext::STAGE_RECORD_CHAPTERS as i64 + (chapter as i64) * 0xd0) as usize;
        let key = ctx.block_at::<4>(record_at + 0xcc)?;
        let raw = ctx.block_at::<4>(record_at + stage_row as i64 as usize * 4)?;

        if u32::from_le_bytes([raw[0] ^ key[0], raw[1] ^ key[1], raw[2] ^ key[2], raw[3] ^ key[3]]) as i32 >= 0x2710 {
            ctx.set_block_at::<4>(record_at + stage_row as i64 as usize * 4, [key[0] ^ 0xf, key[1] ^ 0x27, key[2], key[3]])?;
        }

        let map_type = get_map_type(ctx, 0)?;
        let map_index = get_map_index(ctx, 0)?;
        let stage = get_stage_index(ctx)?;

        if get_stage_record(ctx, map_type, map_index, stage, 0, 0)? == 1
            && (map_type.wrapping_add(7) as u32) <= 5
            && 0x31u32 >> map_type.wrapping_add(7) & 1 != 0
            && get_stage_index(ctx)? == 0x2f
        {
            let map_id = get_global_map_id(ctx, 0)?;

            shop_offer_start(ctx, map_id)?;
        }

        let mut chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;

        if (ctx.i32_at(AppContext::STAGE_ROW)? > 0 && chapter == 0) || chapter > 0 {
            let unlocked = ctx.i32_at(AppContext::RESULT_STAGE_CLEARED)?;

            if unlocked != -1 {
                for unit in 0..0x36cusize {
                    let row = ctx.bytes_from(UNIT_BUY + unit * UNIT_BUY_STRIDE)?;

                    if unit_buy_field(row, 0xf)? != chapter {
                        continue;
                    }

                    let progress_row = ctx.bytes_from(AppContext::CHAPTER_PROGRESS)?;
                    let progress = operation::xor_row_decode(progress_row, 10, chapter as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: chapter as i64, limit: 10 })?;
                    let row = ctx.bytes_from(UNIT_BUY + unit * UNIT_BUY_STRIDE)?;
                    let unlock_stage = unit_buy_field(row, 0)?;

                    if progress != 0x30 {
                        if unlock_stage == unlocked {
                            ctx.set_i32_at(AppContext::UNIT_UNLOCKED_BY_CLEAR, 1)?;

                            break;
                        }
                    } else if unlock_stage == unlocked {
                        if unit_buy_field(row, 1)? == 0 {
                            break;
                        }

                        ctx.set_i32_at(AppContext::UNIT_UNLOCKED_BY_CLEAR, 1)?;

                        break;
                    }
                }
            }

            chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
        }

        if invasion_z_available(ctx, chapter)? && ctx.u8_at(AppContext::BATTLE_IS_Z_INVASION)? != 0 {
            ctx.set_block_at::<1>(AppContext::INVASION_STAGE, [0xff])?;
            ctx.set_block_at::<1>(AppContext::MAP_NEG25_CLEARED, [1])?;
            analytics_named(ctx, 1, b"", b"")?;
            analytics_named(ctx, 0x21, b"", b"")?;
        }
    } else {
        let ex_map = ctx.i32_at(AppContext::EX_MAP)?;

        pair_stage = ctx.i32_at(AppContext::EX_STAGE)?;
        ctx.set_block_at::<1>(AppContext::EX_OFFERED, [1])?;
        ctx.set_i32_at(AppContext::NEXT_STAGE_UNLOCKED, -1)?;

        if ex_map as u32 <= 0x51 {
            let entry = ctx.i32_at(AppContext::ENTRY_STAGE)?;

            if entry as u32 <= 0xb {
                let cell = (ex_map as i64 * 0x30 + entry as i64 * 4 + AppContext::STAGE_RECORD_NEG8 as i64) as usize;
                let count = ctx.i32_at(cell)?;

                if count <= 0x270e {
                    ctx.set_i32_at(cell, count.wrapping_add(1))?;
                }
            }
        }

        if ex_redirect_check_a(ctx)? {
            ctx.set_block_at::<1>(AppContext::EX_REDIRECT_A_BLOCKED, [1])?;
        } else if is_aku_final_map(ctx)? {
            let map_id = get_global_map_id(ctx, 0)?;

            shop_offer_start(ctx, map_id)?;
            ctx.set_i32_at(AppContext::RESULT_MAP_LOCKED, 1)?;
        } else if is_ex_option_target(ctx)? {
            clear_ex_replacement_stage(ctx)?;
        }

        let ex_now = ctx.i32_at(AppContext::EX_MAP)?;
        let entry = ctx.i32_at(AppContext::ENTRY_STAGE)?;

        log_analytics_event(ctx, 5, ex_now, entry, 0, 0)?;

        let ex_now = ctx.i32_at(AppContext::EX_MAP)?;

        mission_progress(ctx, 0, ex_now.wrapping_add(0xfa0), 1, cleared as i32, 0)?;

        let ex_now = ctx.i32_at(AppContext::EX_MAP)?;
        let entry = ctx.i32_at(AppContext::ENTRY_STAGE)?;

        mission_progress(ctx, 1, ex_now.wrapping_mul(0x64).wrapping_add(entry).wrapping_add(0x61a80), 1, cleared as i32, 0)?;
        mission_progress(ctx, 9, 4, 1, cleared as i32, 0)?;

        let ex_now = ctx.i32_at(AppContext::EX_MAP)?;
        let bits = map_guerrilla_set(ctx, ex_now.wrapping_add(0xfa0));

        pair_map = ex_map.wrapping_add(0xfa0);

        if bits > 0 {
            let mut bit = 0i32;
            let mut mask = 1i32;

            loop {
                if mask & bits != 0 {
                    mission_progress(ctx, 0x14, bit, 1, cleared as i32, 0)?;
                }

                mask = 2i32.wrapping_shl(bit as u32 & 0x1f);
                bit += 1;

                if mask > bits {
                    break;
                }
            }
        }
    }

    if let Some(pair) = stage_pair_record(ctx, map_key, pair_stage)?
        && pair.other_stage > stage_pair_progress(ctx, map_key)
    {
        ctx.reward_queue.push(vec![0xa, map_key, pair_stage]);

        let other = stage_pair_record(ctx, map_key, pair_stage)?.ok_or(Fault::NullPointer { site: SITE })?.other_stage;

        stage_pair_progress_set(ctx, map_key, pair_stage, other);
    }

    let star = ctx.i32_at(AppContext::STAR_LEVEL)?;

    if get_stage_record(ctx, map_type_of_map_id(pair_map), map_index_of_map_id(pair_map), pair_stage, star, use_cache)? == 1 && altar_stage_value(ctx, pair_map, pair_stage) > 0 {
        ctx.reward_queue.push(vec![9]);
    }

    let row = (AppContext::MAP_STAGE_ROWS as i64 + (pair_stage as i64) * AppContext::MAP_STAGE_ROW_STRIDE as i64) as usize;
    let selector = xor_row46_get(ctx.bytes_from(row)?, 8).ok_or(Fault::IndexOutOfRange { site: SITE, index: 8, limit: 0x2e })?;

    'drops: {
        if selector & 0xfffffffe != 0xfffffffc {
            let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
            let outbreak_chapter = (chapter.wrapping_sub(4) as u32) <= 2 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? == 0;

            if outbreak_chapter
                || xor_row46_get(ctx.bytes_from(row)?, 5).ok_or(Fault::IndexOutOfRange { site: SITE, index: 5, limit: 0x2e })? == 0xffffffff
                || ctx.u8_at(AppContext::BATTLE_IS_INVASION)? != 0
            {
                if invasion_available(ctx, chapter)? && ctx.u8_at(AppContext::BATTLE_IS_INVASION)? != 0 {
                    ctx.set_block_at::<1>(AppContext::MAP_NEG15_CLEARED, [1])?;
                    roll_filibuster_stage(ctx)?;
                    analytics_named(ctx, 1, b"10", "\u{30d5}\u{30a3}\u{30ea}\u{30d0}\u{30b9}\u{30bf}\u{30fc}".as_bytes())?;
                    analytics_named(ctx, 0x21, b"10", "\u{30d5}\u{30a3}\u{30ea}\u{30d0}\u{30b9}\u{30bf}\u{30fc}".as_bytes())?;

                    if event_unit_slot(&ctx.event_unit_rows, 0x1ce)? != -1 {
                        let slot = event_unit_slot(&ctx.event_unit_rows, 0x1ce)?;

                        ctx.set_i32_at(((slot as i64) * 4 + AppContext::EVENT_UNIT_OWNED as i64) as usize, 1)?;
                    }

                    ctx.set_i32_at(AppContext::UNITS_UNLOCKED_FLAG, 1)?;
                    ctx.reward_queue.push(vec![7]);
                }

                break 'drops;
            }

            let mut tiers = 1;

            if (selector as i32) >= 0 && (xor_row46_get(ctx.bytes_from(row)?, 9).ok_or(Fault::IndexOutOfRange { site: SITE, index: 9, limit: 0x2e })? as i32) >= 0 {
                tiers = ((xor_row46_get(ctx.bytes_from(row)?, 0xc).ok_or(Fault::IndexOutOfRange { site: SITE, index: 0xc, limit: 0x2e })? as i32) >= 0) as i32 | 2;
            }

            ctx.set_i32_at(AppContext::RANK_REWARD_BASE, 0)?;

            let mut tier = 0;

            loop {
                ctx.set_i32_at(AppContext::DROP_FLAG, 0)?;

                let rate_field = match tier {
                    0 => Some((5usize, 0)),
                    1 => Some((9, 4)),
                    2 => Some((0xc, 7)),
                    _ => None,
                };

                if let Some((field, base)) = rate_field {
                    let rate = xor_row46_get(ctx.bytes_from(row)?, field).ok_or(Fault::IndexOutOfRange { site: SITE, index: field as i64, limit: 0x2e })? as i32;

                    ctx.set_i32_at(AppContext::DROP_RATE, rate)?;
                    ctx.set_i32_at(AppContext::RANK_REWARD_BASE, base)?;
                }

                let roll = call_rng(ctx, 0x64);

                ctx.set_i32_at(AppContext::DROP_ROLL, roll)?;

                if get_powerup(ctx, 1)? && (ctx.i32_at(AppContext::CHAPTER_MODE)? == 3 || is_aku_final_map(ctx)?) {
                    ctx.set_i32_at(AppContext::DROP_RATE, 0x64)?;
                }

                if ctx.i32_at(AppContext::DROP_RATE)? > ctx.i32_at(AppContext::DROP_ROLL)? {
                    ctx.set_block_at::<1>(AppContext::RANK_POPUP_SHOWN, [0])?;

                    let (grant, group_was_cleared) = 'decide: {
                        if ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0 {
                            if tier != 0 {
                                break 'decide (true, true);
                            }
                        } else if tier != 0 || ctx.i32_at(AppContext::EVENT_REWARD_ID)? == -1 {
                            break 'decide (true, true);
                        }

                        let kind = xor_row46_get(ctx.bytes_from(row)?, 8).ok_or(Fault::IndexOutOfRange { site: SITE, index: 8, limit: 0x2e })?;

                        if kind.wrapping_sub(1) > 1 {
                            break 'decide (true, true);
                        }

                        if ctx.i32_at(AppContext::EVENT_REWARD_ID)? != -1 {
                            let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
                            let event_chapter = chapter == 0x63 || chapter == 3;

                            if event_chapter {
                                let reward_kind = stage_reward_kind(ctx, pair_stage, 0)?;
                                let unseen = reward_kind == 0 && {
                                    let event = ctx.i32_at(AppContext::EVENT_REWARD_ID)?;
                                    let star = get_star_level(ctx)?;
                                    let kind = xor_row46_get(ctx.bytes_from(row)?, 8).ok_or(Fault::IndexOutOfRange { site: SITE, index: 8, limit: 0x2e })? as i32;

                                    !event_reward_received(ctx, event, pair_stage, star, kind, use_cache)?
                                };

                                if unseen {
                                    let event = ctx.i32_at(AppContext::EVENT_REWARD_ID)?;
                                    let star = get_star_level(ctx)?;
                                    let kind = xor_row46_get(ctx.bytes_from(row)?, 8).ok_or(Fault::IndexOutOfRange { site: SITE, index: 8, limit: 0x2e })? as i32;

                                    event_reward_set(ctx, event, pair_stage, star, kind, 1, use_cache)?;

                                    break 'decide (true, true);
                                }
                            }

                            if ctx.i32_at(AppContext::EVENT_REWARD_ID)? != -1 && event_chapter {
                                let kind = stage_reward_kind(ctx, pair_stage, 0)?;

                                if (kind == 1 || stage_reward_kind(ctx, pair_stage, 0)? == 2 || stage_reward_kind(ctx, pair_stage, 0)? == 3) && {
                                    let item = stage_reward_item(ctx, pair_stage, 0)?;

                                    !reward_owned(ctx, item)?
                                } {
                                    let event = ctx.i32_at(AppContext::EVENT_REWARD_ID)?;
                                    let star = get_star_level(ctx)?;
                                    let kind = xor_row46_get(ctx.bytes_from(row)?, 8).ok_or(Fault::IndexOutOfRange { site: SITE, index: 8, limit: 0x2e })? as i32;

                                    event_reward_set(ctx, event, pair_stage, star, kind, 1, use_cache)?;

                                    break 'decide (true, true);
                                }
                            }
                        }

                        let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;

                        if chapter != 3 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0 {
                            let seen = ctx.outbreak_cleared.entry(chapter).or_default().get(&pair_stage).copied();

                            if seen != Some(true) {
                                ctx.outbreak_cleared.entry(chapter).or_default().entry(pair_stage).or_insert(false);

                                let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
                                let group = treasure_group_of_stage(ctx, chapter, pair_stage)?;
                                let was = treasure_group_cleared(ctx, chapter, group)?;
                                let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;

                                *ctx.outbreak_cleared.entry(chapter).or_default().entry(pair_stage).or_insert(false) = true;

                                let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;

                                *ctx.outbreak_active.entry(chapter).or_default().entry(pair_stage).or_insert(false) = false;

                                let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;

                                log_analytics_event(ctx, 0x1f, chapter, pair_stage, 0, 0)?;

                                break 'decide (true, was);
                            }
                        }

                        (false, true)
                    };

                    if grant {
                        let status = grant_stage_reward(ctx, ctx.i32_at(AppContext::RANK_REWARD_BASE)?.wrapping_add(6), 0)?;

                        ctx.set_i32_at(AppContext::REWARD_STATUS, status)?;

                        if status != 1 {
                            let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;

                            if chapter == 3 || ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? == 0 {
                                let base = ctx.i32_at(AppContext::RANK_REWARD_BASE)?;

                                ctx.reward_queue.push(vec![0, base.wrapping_add(6), 0, status]);

                                if stage_reward_kind(ctx, pair_stage, 0)? == 1 && ctx.i32_at(AppContext::REWARD_STATUS)? == 2 {
                                    let item = stage_reward_item(ctx, pair_stage, 0)?;
                                    let unit = reward_unit_id(ctx, item)?;

                                    log_analytics_event(ctx, 0x5e, unit, 0, 0, 0)?;
                                }
                            } else {
                                let rank = if chapter >= 3 { (chapter >= 7) as i32 + 1 } else { 0 };

                                ctx.reward_queue.push(vec![3, rank, pair_stage]);

                                if !group_was_cleared {
                                    let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
                                    let group = treasure_group_of_stage(ctx, chapter, pair_stage)?;

                                    if treasure_group_cleared(ctx, chapter, group)? {
                                        let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
                                        let group = treasure_group_of_stage(ctx, chapter, pair_stage)?;

                                        ctx.reward_queue.push(vec![4, rank, group]);

                                        if pair_stage == 0x2f {
                                            ctx.reward_queue.push(vec![5, rank]);
                                        }
                                    }
                                }
                            }
                        }

                        ctx.set_i32_at(AppContext::DROP_FLAG, 1)?;

                        break 'drops;
                    }
                }

                if get_powerup(ctx, 1)? {
                    ctx.set_block_at::<1>(AppContext::RANK_POPUP_SHOWN, [1])?;
                    ctx.set_i32_at(AppContext::DROP_FLAG, 1)?;

                    break 'drops;
                }

                tier += 1;

                if tier == tiers {
                    break 'drops;
                }
            }
        }

        let mut rate = ctx.i32_at(AppContext::DROP_RATE)?;
        let mut count = 0i32;
        let mut field = 5i32;

        loop {
            let slot = if count == 0 { field } else { field + 1 };
            let chance = xor_row46_get(ctx.bytes_from(row)?, slot as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: slot as i64, limit: 0x2e })?;

            if chance == 0xffffffff {
                break;
            }

            rate = rate.wrapping_add(chance as i32);
            ctx.set_i32_at(AppContext::DROP_RATE, rate)?;
            count += 1;
            field += 3;

            if count == 0xb {
                count = 0;

                break;
            }
        }

        let pick = call_rng(ctx, rate);

        ctx.set_i32_at(AppContext::DROP_ROLL, pick)?;
        ctx.set_i32_at(AppContext::DROP_RATE, 0)?;

        if count == 0 {
            break 'drops;
        }

        let mut sum = 0i32;
        let mut step = 0i32;

        while step != count.wrapping_mul(3) {
            let base = if step == 0 { 0 } else { step + 1 };
            let chance = xor_row46_get(ctx.bytes_from(row)?, (base + 5) as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: base as i64 + 5, limit: 0x2e })?;

            sum = sum.wrapping_add(chance as i32);
            ctx.set_i32_at(AppContext::DROP_RATE, sum)?;
            ctx.set_i32_at(AppContext::RANK_REWARD_BASE, base)?;

            if pick < sum {
                ctx.set_block_at::<1>(AppContext::RANK_POPUP_SHOWN, [0])?;

                let taken = stage_reward_taken(ctx, pair_map, pair_stage);
                let kind = xor_row46_get(ctx.bytes_from(row)?, 8).ok_or(Fault::IndexOutOfRange { site: SITE, index: 8, limit: 0x2e })?;

                if taken || kind != 0xfffffffd {
                    if kind != 0xfffffffc {
                        if !get_powerup(ctx, 1)? {
                            ctx.set_i32_at(AppContext::DROP_RATE, 0)?;
                        } else {
                            ctx.set_block_at::<1>(AppContext::RANK_POPUP_SHOWN, [1])?;
                            ctx.set_i32_at(AppContext::DROP_FLAG, 1)?;
                        }

                        break 'drops;
                    }
                } else {
                    *ctx.stage_rewards_taken.entry(pair_map).or_default().entry(pair_stage).or_insert(false) = true;
                }

                let base = ctx.i32_at(AppContext::RANK_REWARD_BASE)?;
                let status = grant_stage_reward(ctx, base.wrapping_add(6), 0)?;

                ctx.set_i32_at(AppContext::REWARD_STATUS, status)?;

                let base = ctx.i32_at(AppContext::RANK_REWARD_BASE)?;

                ctx.reward_queue.push(vec![0, base.wrapping_add(6), 0, status]);
                ctx.set_i32_at(AppContext::DROP_FLAG, 1)?;

                break 'drops;
            }

            step += 3;
        }
    }

    if ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? == 0 && cleared == 0 {
        for status in 0..10usize {
            ctx.set_i32_at(AppContext::SCORE_RANK_STATUSES + status * 4, -1)?;
        }

        if !is_score_stage(ctx.event_items.as_ref()) {
            ctx.set_i32_at(AppContext::NEW_BEST_SCORE, 0)?;
        }

        if xor_row46_get(ctx.bytes_from(row)?, 0xf).ok_or(Fault::IndexOutOfRange { site: SITE, index: 0xf, limit: 0x2e })? == 1 {
            let group = ctx.i32_at(AppContext::CHAPTER_MODE)?.wrapping_sub(4);
            let ranking = ctx.i32_at(AppContext::RANKING_ID)?;

            if (group as u32) < 3 || ranking != -1 {
                let stage_row = ctx.i32_at(AppContext::STAGE_ROW)? as i64;
                let best = if (group as u32) <= 2 {
                    ctx.i32_at((group as u32 as i64 * 0xcc + stage_row * 4 + AppContext::CHAPTER_BEST_SCORES as i64) as usize)?
                } else if ranking == -1 {
                    0
                } else {
                    let star = ctx.i32_at(AppContext::STAR_LEVEL)? as i64;

                    ctx.i32_at((stage_row * 0x10 + ranking as i64 * 0xf0 + star * 4 + AppContext::RANKING_BEST_SCORES as i64) as usize)?
                };
                let frames = ctx.i32_at(AppContext::PLAY_FRAMES)?;
                let score = if frames > 0x2328 {
                    if frames as u32 <= 0x4a37 { 0x3e8 - ((frames as u16).wrapping_sub(0x2328) / 10) as i32 } else { 0 }
                } else {
                    0x2710i32.wrapping_sub(frames)
                };

                ctx.set_i32_at(AppContext::STAGE_SCORE, score)?;

                if score > best {
                    let stage_row = ctx.i32_at(AppContext::STAGE_ROW)?;
                    let stage_at = (AppContext::MAP_STAGE_ROWS as i64 + (stage_row as i64) * AppContext::MAP_STAGE_ROW_STRIDE as i64) as usize;
                    let mut ranks = 9i64;

                    for (index, field) in [0x13usize, 0x16, 0x19, 0x1c, 0x1f, 0x22, 0x25, 0x28].into_iter().enumerate() {
                        if xor_row46_get(ctx.bytes_from(stage_at)?, field).ok_or(Fault::IndexOutOfRange { site: SITE, index: field as i64, limit: 0x2e })? == 0xffffffff {
                            ranks = index as i64 + 1;

                            break;
                        }

                        if index == 7 {
                            ranks = 9 + (xor_row46_get(ctx.bytes_from(stage_at)?, 0x2b).ok_or(Fault::IndexOutOfRange { site: SITE, index: 0x2b, limit: 0x2e })? != 0xffffffff) as i64;
                        }
                    }

                    let mut step = 0i64;
                    let mut slot = 0usize;

                    while step != ranks * 3 {
                        let threshold = xor_row46_get(ctx.bytes_from(row)?, (0x10 + step) as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: 0x10 + step, limit: 0x2e })? as i32;

                        if ctx.i32_at(AppContext::STAGE_SCORE)? >= threshold && threshold > best {
                            let status = grant_stage_reward(ctx, step as i32 + 0x11, 1)?;

                            ctx.set_i32_at(AppContext::SCORE_RANK_STATUSES + slot * 4, status)?;

                            if status != 1 {
                                ctx.reward_queue.push(vec![0, step as i32 + 0x11, 1, status]);
                            }
                        }

                        step += 3;
                        slot += 1;
                    }

                    let group = ctx.i32_at(AppContext::CHAPTER_MODE)?.wrapping_sub(4);

                    if (group as u32) > 2 {
                        let ranking = ctx.i32_at(AppContext::RANKING_ID)?;

                        if ranking != -1 {
                            let stage_row = ctx.i32_at(AppContext::STAGE_ROW)? as i64;
                            let star = ctx.i32_at(AppContext::STAR_LEVEL)? as i64;

                            ctx.set_i32_at((stage_row * 0x10 + ranking as i64 * 0xf0 + star * 4 + AppContext::RANKING_BEST_SCORES as i64) as usize, ctx.i32_at(AppContext::STAGE_SCORE)?)?;
                        }
                    } else {
                        let stage_row = ctx.i32_at(AppContext::STAGE_ROW)? as i64;

                        ctx.set_i32_at((group as u32 as i64 * 0xcc + stage_row * 4 + AppContext::CHAPTER_BEST_SCORES as i64) as usize, ctx.i32_at(AppContext::STAGE_SCORE)?)?;
                    }

                    ctx.set_i32_at(AppContext::NEW_BEST_SCORE, 1)?;
                }

                let ranking = ctx.i32_at(AppContext::RANKING_ID)?;
                let stage_row = ctx.i32_at(AppContext::STAGE_ROW)?;
                let star = ctx.i32_at(AppContext::STAR_LEVEL)?;
                let score = ctx.i32_at(AppContext::STAGE_SCORE)?;

                mission_progress(ctx, 0x12, ranking.wrapping_mul(0x3e8).wrapping_add(stage_row.wrapping_mul(10)).wrapping_add(star), score, 0, 0)?;
            }
        }
    }

    let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;

    if chapter != 3 && chapter != 0x63 {
        if ctx.i32_at(AppContext::FIRST_STAGE_WON)? | ctx.i32_at(AppContext::STAGE_ROW)? == 0 {
            ctx.set_i32_at(AppContext::FIRST_STAGE_WON, 1)?;
        }

        ctx.set_i32_at(AppContext::WIN_TREASURE, 0)?;
        ctx.set_i32_at(AppContext::NEXT_STAGE_UNLOCKED, -1)?;
        ctx.set_i32_at(AppContext::DROP_FLAG, 0)?;
        ctx.set_i32_at(AppContext::DROP_RATE, 0)?;

        let area = config_json_int(ctx, b"CnfJsonGetTreasureRewrite", b"area", 1, 9)?;
        let matched = treasure_area_match(ctx.i32_at(AppContext::CHAPTER_MODE)?, area);
        let (mut treasure, mut zombie, mut festival) = (0, 0, 0);

        if matched {
            treasure = config_json_int(ctx, b"CnfJsonGetTreasureRewrite", b"treasure", 0x23, 0x64)?;
            zombie = config_json_int(ctx, b"CnfJsonGetTreasureRewrite", b"zombie", 0x32, 0x64)?;
            festival = config_json_int(ctx, b"CnfJsonGetTreasureRewrite", b"festival", 0x46, 0x64)?;
        }

        if !matched || treasure >= zombie || zombie >= festival {
            treasure = 0x23;
            zombie = 0x32;
            festival = 0x46;
        }

        ctx.set_i32_at(AppContext::DROP_RATE, treasure)?;

        let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
        let group = treasure_group_of_stage(ctx, chapter, pair_stage)?;

        if treasure_group_cleared(ctx, chapter, group)? {
            ctx.set_i32_at(AppContext::DROP_RATE, zombie)?;
        }

        if ctx.u8_at(AppContext::TREASURE_FESTIVAL_ENABLED)? != 0 {
            let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
            let progress_row = ctx.bytes_from(AppContext::CHAPTER_PROGRESS)?;
            let progress = operation::xor_row_decode(progress_row, 10, chapter as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: chapter as i64, limit: 10 })? as i32;

            if progress >= 0x19
                && ctx.i32_at(AppContext::FIRST_STAGE_WON)? > 0
                && ctx.i32_at(AppContext::STAGE_ROW)? == ctx.i32_at(((chapter as i64) * 4 + AppContext::FESTIVAL_STAGES as i64) as usize)?
            {
                ctx.set_i32_at(AppContext::DROP_RATE, festival)?;
            }
        }

        if treasure_festival_active(ctx)? {
            ctx.set_i32_at(AppContext::DROP_RATE, festival)?;
        }

        if get_powerup(ctx, 1)? {
            ctx.set_i32_at(AppContext::DROP_RATE, 0x64)?;
        }

        if get_global_map_id(ctx, 0)? == 0xbb8 && get_stage_index(ctx)? <= 2 {
            ctx.set_i32_at(AppContext::DROP_RATE, 0x64)?;
        }

        let roll = call_rng(ctx, 0x64);

        ctx.set_i32_at(AppContext::DROP_ROLL, roll)?;

        if ctx.i32_at(AppContext::DROP_RATE)? > roll {
            let grade = call_rng(ctx, 0x64);

            ctx.set_i32_at(AppContext::DROP_FLAG, grade)?;

            let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
            let castle = ctx.i32_at(AppContext::CASTLE_ID)?;
            let levels = ctx.bytes_from(AppContext::TREASURE_LEVELS + chapter as i64 as usize * AppContext::TREASURE_LEVELS_STRIDE)?;
            let level = operation::xor_row_decode(levels, 0x31, castle as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: castle as i64, limit: 0x31 })?;
            let tier = match level {
                1 => ((grade < 0x46) as i32) ^ 3,
                0 => {
                    if grade >= 0x2d {
                        3 - (grade < 0x4b) as i32
                    } else {
                        1
                    }
                }
                _ => 3,
            };

            ctx.set_i32_at(AppContext::WIN_TREASURE, tier)?;

            if get_powerup(ctx, 1)? {
                let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
                let castle = ctx.i32_at(AppContext::CASTLE_ID)?;
                let levels = ctx.bytes_from(AppContext::TREASURE_LEVELS + chapter as i64 as usize * AppContext::TREASURE_LEVELS_STRIDE)?;
                let level = operation::xor_row_decode(levels, 0x31, castle as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: castle as i64, limit: 0x31 })?;
                let tier = if level == 3 || (get_global_map_id(ctx, 0)? == 0xbb8 && get_stage_index(ctx)? <= 2) {
                    ctx.set_block_at::<1>(AppContext::RANK_POPUP_SHOWN, [1])?;
                    ctx.set_i32_at(AppContext::DROP_FLAG, 1)?;

                    0
                } else {
                    3
                };

                ctx.set_i32_at(AppContext::WIN_TREASURE, tier)?;
            }

            if ctx.i32_at(AppContext::FIRST_STAGE_WON)? == 1 {
                ctx.set_i32_at(AppContext::FIRST_STAGE_WON, 2)?;
            }

            if get_global_map_id(ctx, 0)? == 0xbb8 && get_stage_index(ctx)? <= 2 {
                ctx.set_i32_at(AppContext::WIN_TREASURE, 3)?;
            }

            let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
            let castle = ctx.i32_at(AppContext::CASTLE_ID)?;
            let levels_at = AppContext::TREASURE_LEVELS + chapter as i64 as usize * AppContext::TREASURE_LEVELS_STRIDE;
            let level = operation::xor_row_decode(ctx.bytes_from(levels_at)?, 0x31, castle as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: castle as i64, limit: 0x31 })? as i32;
            let tier = ctx.i32_at(AppContext::WIN_TREASURE)?;

            if level < tier {
                let key = ctx.block_at::<4>(levels_at + 0xc4)?;
                let value = tier.to_le_bytes();

                ctx.set_block_at::<4>(levels_at + castle as i64 as usize * 4, [key[0] ^ value[0], key[1] ^ value[1], key[2] ^ value[2], key[3] ^ value[3]])?;
                ctx.reward_queue.push(vec![1]);

                let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
                let groups = ctx.treasure_store.get(chapter as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: chapter as i64, limit: 10 })?.len() as i32;

                if groups > 0 {
                    let mut group = 0i32;

                    'search: loop {
                        let mut index = 0i64;

                        loop {
                            let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
                            let record = treasure_group_at(&ctx.treasure_store, chapter, group)?;

                            if index >= record.count as i64 {
                                group += 1;

                                let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
                                let groups = ctx.treasure_store.get(chapter as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: chapter as i64, limit: 10 })?.len() as i32;

                                if group < groups {
                                    continue 'search;
                                }

                                break 'search;
                            }

                            let castle = *record.castles.get(index as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index, limit: record.castles.len() as i64 })?;

                            index += 1;

                            if castle == ctx.i32_at(AppContext::CASTLE_ID)? {
                                break;
                            }
                        }

                        let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
                        let progress = ((chapter as i64) * 0x2c + (group as i64) * 4 + AppContext::TREASURE_PROGRESS as i64) as usize;

                        if ctx.i32_at(progress)? == 0 {
                            calculate_treasure_percentages(ctx)?;

                            let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
                            let progress = ((chapter as i64) * 0x2c + (group as i64) * 4 + AppContext::TREASURE_PROGRESS as i64) as usize;

                            if ctx.i32_at(progress)? == 0 {
                                calculate_treasure_percentages(ctx)?;
                            } else {
                                ctx.set_i32_at(AppContext::NEXT_STAGE_UNLOCKED, group)?;
                            }
                        }

                        break 'search;
                    }
                }

                ctx.set_i32_at(AppContext::DROP_RATE, 0)?;
                ctx.set_i32_at(AppContext::DROP_ROLL, 0x64)?;
            }
        }
    }

    if get_map_type(ctx, 0)? == -6 {
        let frames = ctx.i32_at(AppContext::PLAY_FRAMES)?;
        let gap = 0x7530i32.wrapping_sub(frames);
        let score = if frames < 0x7530 { gap.wrapping_mul(gap) } else { 0 };

        ctx.set_i32_at(AppContext::SCORE_TOTAL, score)?;

        let stage = get_stage_index(ctx)? as i64;

        if score > ctx.i32_at((stage * 4 + AppContext::SPECIAL_BEST_SCORES as i64) as usize)? {
            let score = ctx.i32_at(AppContext::SCORE_TOTAL)?;
            let stage = get_stage_index(ctx)? as i64;

            ctx.set_i32_at((stage * 4 + AppContext::SPECIAL_BEST_SCORES as i64) as usize, score)?;
        }
    }

    let star = ctx.i32_at(AppContext::STAR_LEVEL)?;
    let mut drop_map = pair_map;
    let mut drop_stage = pair_stage;

    if get_map_type(ctx, 0)? == -0xb {
        let stage = get_stage_index(ctx)? as i64;
        let packed = ctx.i32_at((stage * 4 + AppContext::DROP_MAP_STAGES as i64) as usize)?;
        let stage = get_stage_index(ctx)? as i64;
        let again = ctx.i32_at((stage * 4 + AppContext::DROP_MAP_STAGES as i64) as usize)?;

        drop_map = packed / 100;
        drop_stage = again.wrapping_sub((again / 100).wrapping_mul(100));
    }

    let mut counts = roll_drop_item_counts(ctx, drop_map, drop_stage, star)?;

    'items: for index in 0..counts.len() {
        if *counts.get(index).ok_or(Fault::IndexOutOfRange { site: SITE, index: index as i64, limit: 0x10 })? == 0 {
            continue;
        }

        for item in 0..counts.len() {
            let count = *counts.get(item).ok_or(Fault::IndexOutOfRange { site: SITE, index: item as i64, limit: 0x10 })?;

            for _ in 0..count.max(0) {
                ctx.item_drop_queue.push(vec![item as i32]);
            }
        }

        counts.insert(0, 6);
        ctx.reward_queue.push(counts.clone());

        if counts.len() >= 2 {
            let mut granted = 0i32;

            for item in 1..counts.len() {
                let amount = *counts.get(item).ok_or(Fault::IndexOutOfRange { site: SITE, index: item as i64, limit: 0x11 })?;

                if amount != 0 {
                    if granted > 3 {
                        break 'items;
                    }

                    let kind = find_item_by_kind(ctx, 7, item as i32 - 1)?;

                    add_resource(ctx, kind, amount, 0)?;
                    granted += 1;
                }
            }
        }

        break;
    }

    let map_type = get_map_type(ctx, 0)?;

    if ctx.i32_at(AppContext::CHAPTER_MODE)? == 3
        && ctx.u8_at(AppContext::SCORE_MODE_FLAG)? == 0
        && ((map_type.wrapping_add(0x13) as u32) > 0xd || 0x2205u32 >> map_type.wrapping_add(0x13) & 1 == 0)
    {
        enigma_prune(ctx)?;

        let stage = ctx.i32_at(AppContext::STAGE_INDEX)?;
        let halve = ctx.u8_at(AppContext::STAMINA_HALVED)?;
        let cost = stage_stamina_cost(ctx, stage, halve)?;

        enigma_add_stamina(ctx, cost)?;

        if enigma_roll(ctx)? != 0 {
            let last = enigma_active_at(&ctx.enigma, (enigma_active_count(&ctx.enigma) - 1) as i32)?.group;
            let id = enigma_group_at(&ctx.enigma, last)?.id;

            ctx.reward_queue.push(vec![8, id]);

            let last = enigma_active_at(&ctx.enigma, (enigma_active_count(&ctx.enigma) - 1) as i32)?.group;
            let medals = enigma_medal_count(ctx)?;

            log_analytics_event(ctx, 0x4f, last, medals, 0, 0)?;
        }
    }

    if ctx.i32_at(AppContext::CHAPTER_MODE)? == 3 && ctx.i32_at(AppContext::NEXT_STAGE_UNLOCKED)? == 0 {
        ctx.reward_queue.push(vec![0xb]);
        ctx.set_i32_at(AppContext::NEXT_STAGE_UNLOCKED, -1)?;
    }

    if replay {
        let lineups = clear_lineup_count(ctx, -1, 0, 0)?;
        let map_type = validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?);
        let map_index = ctx.i32_at(AppContext::MAP_INDEX)?;
        let stage_row = ctx.i32_at(AppContext::STAGE_ROW)?;
        let star = ctx.i32_at(AppContext::STAR_LEVEL)?;
        let record = get_stage_record(ctx, map_type, map_index, stage_row, star, 1)?;
        let map_id = get_global_map_id(ctx, 0)?;
        let stage = get_stage_index(ctx)?;
        let rewards = clear_count_rewards_get(ctx, map_id, stage);
        let mut item_index = -1;

        if (rewards.len() as u64) > lineups as i64 as u64 {
            let item = rewards.get(lineups as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: lineups as i64, limit: rewards.len() as i64 })?[0];

            if item != -1 {
                item_index = find_item_index(ctx, item)?;
            }
        }

        let mut last = rewards.len() as i32;
        let mut cursor = last.wrapping_sub(1);

        loop {
            if last <= 0 {
                last = -1;

                if !first_record {
                    break;
                }
            } else {
                let at = cursor;

                last -= 1;
                cursor -= 1;

                if rewards.get(at as u32 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: at as i64, limit: rewards.len() as i64 })?[0] == -1 {
                    continue;
                }

                if !first_record {
                    break;
                }
            }

            if record.wrapping_sub(1) > last {
                break;
            }

            if item_index != -1 {
                let amount = rewards.get(lineups as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: lineups as i64, limit: rewards.len() as i64 })?[1];

                add_resource(ctx, item_index, amount, 0)?;
                ctx.reward_queue.push(vec![0xc, lineups, item_index, amount]);
            }

            if record.wrapping_sub(1) == last {
                ctx.reward_queue.push(vec![0xd]);
            }

            break;
        }

        match lineups {
            9 => ctx.reward_queue.push(vec![0x10]),
            0 => ctx.reward_queue.push(vec![0xe]),
            1..=8 => ctx.reward_queue.push(vec![0xf]),
            _ => {}
        }

        clear_lineup_record(ctx, -1, 0, 0)?;
    }

    if is_score_stage(ctx.event_items.as_ref()) {
        let store = ctx.event_items.as_ref().ok_or(Fault::NullPointer { site: SITE })?;
        let point_id = get_point_id(store);
        let total = get_point_total(store);
        let rewards: Vec<(i32, i32, i32, i32, i32)> = get_point_rewards(&ctx.reward_defs, point_id)
            .map(|list| list.iter().map(|reward| (reward.id, reward.threshold, reward.kind, reward.target, reward.amount)).collect())
            .unwrap_or_default();
        let cap = get_release_point_cap(ctx, point_id)?;

        for (id, threshold, kind, target, amount) in rewards {
            if total < threshold || is_reward_claimed(&ctx.reward_claimed, point_id, id) {
                continue;
            }

            set_reward_claimed(&mut ctx.reward_claimed, point_id, id, 1);
            ctx.reward_queue.push(vec![0x11, point_id, id]);

            match kind {
                1 if event_unit_slot(&ctx.event_unit_rows, target)? != -1 => {
                    let slot = event_unit_slot(&ctx.event_unit_rows, target)?;

                    ctx.set_i32_at(((slot as i64) * 4 + AppContext::EVENT_UNIT_OWNED as i64) as usize, 1)?;
                }
                0 => add_resource(ctx, target, amount, 0)?,
                _ => {}
            }

            point_reward_analytics(ctx, point_id, id, (cap <= total) as i32)?;
        }
    }

    if ctx.u8_at(AppContext::RANK_POPUP_SHOWN)? != 0 {
        ctx.set_block_at::<1>(AppContext::RANK_POPUP_SHOWN, [0])?;
        add_resource(ctx, 1, 1, 0)?;
        ctx.reward_queue.push(vec![2]);
    }

    if ctx.i32_at(AppContext::TUTORIAL_CLEARED)? == 0 {
        log_analytics_event(ctx, 0, 0, 0, 0, 0)?;
    }

    let map_id = get_global_map_id(ctx, 0)?;
    let stage_row = ctx.i32_at(AppContext::STAGE_ROW)?;

    if web_popup_stage_match(ctx, map_id, stage_row) {
        web_popup_request(ctx, 3);
    }

    ctx.set_i32_at(AppContext::TUTORIAL_CLEARED, 1)?;
    ctx.set_i32_at(AppContext::CAT_FOOD_SHOP_ENABLED, 1)?;
    ctx.set_i32_at(AppContext::BATTLE_RESUMED, 0)?;
    ctx.set_i32_at(AppContext::BATTLE_CONTINUED, 0)?;
    record_stage_played(ctx)?;
    request_save_data(ctx)?;

    let notice = ctx.u8_at(AppContext::LEADERSHIP_NOTICE)?;

    notification_schedule(ctx, notice, 1)?;

    let map_id = get_global_map_id(ctx, 0)?;

    if ads_available(ctx)? && map_xp_ad(ctx, map_id) {
        ad_prepare(ctx, 0)?;

        if feature_enabled(ctx, 0x6c)? && get_item_count(ctx, 6)?.wrapping_add(ctx.i32_at(AppContext::WIN_XP)?) <= 0x5f5e0ff && reward_ad_ready(ctx, 0, 0)? {
            ctx.set_block_at::<1>(AppContext::RESULT_VIDEO_BUTTON, [1])?;

            let (id, x, y, width, height, panel) = if cleared == 0 {
                let sheet = Rc::clone(ctx.img039_sheet.as_ref().ok_or(Fault::NullPointer { site: SITE })?);
                let width = get_drawable_width(ctx)?;
                let x = width.wrapping_sub(0xfa) / 2;
                let mut panel = ui_node_set_panel(&sheet, x.wrapping_add(0x7d), 0x19c, 0xfa, 0x36, 0x1a, 0x1b, 1.0);

                ui_node_set_anchor(&mut panel, 1);

                let left = ui_node_set_sprite(&sheet, -0x61, 0, 0x18)?;

                ui_node_set_anchor(ui_node_add_child(&mut panel, left), 1);

                let right = ui_node_set_sprite(&sheet, 0x14, 0, 0x19)?;

                ui_node_set_anchor(ui_node_add_child(&mut panel, right), 1);

                let top = ui_node_set_sprite(&sheet, 0, -0x1c, 0x1c)?;

                ui_node_set_anchor(ui_node_add_child(&mut panel, top), 1);

                (0xcb, x, 0x181, 0xfa, 0x36, panel)
            } else {
                let sheet = Rc::clone(ctx.map_ui_sheet.as_ref().ok_or(Fault::NullPointer { site: SITE })?);
                let dialog = dialog_top(ctx).ok_or(Fault::NullPointer { site: SITE })?;
                let (origin_x, origin_y) = dialog_origin(ctx, dialog)?;
                let mut panel = ui_node_set_panel(&sheet, origin_x.wrapping_add(0x78), origin_y.wrapping_sub(0x1c), 0xe7, 0x32, 0x1a, 0x1b, 1.0);

                ui_node_set_anchor(&mut panel, 1);

                let left = ui_node_set_sprite(&sheet, -0x58, 0, 0x18)?;
                let child = ui_node_add_child(&mut panel, left);

                ui_node_set_anchor(child, 1);
                ui_node_set_zoom(child, 0.94, 0.94);

                let right = ui_node_set_sprite(&sheet, 0x13, 0, 0x19)?;
                let child = ui_node_add_child(&mut panel, right);

                ui_node_set_anchor(child, 1);
                ui_node_set_zoom(child, 0.94, 0.94);

                (0xd, origin_x.wrapping_add(5), origin_y.wrapping_sub(0x35), 0xe7, 0x32, panel)
            };

            ctx.ad_button_cleared = cleared;

            let button = new_button_register(&mut ctx.buttons, id, x, y, width, height, Some(panel), Some(battle_init_win_lambda_0));

            ctx.ad_button_id = button;
            new_button_set_touchable(&mut ctx.buttons, button, 0)?;
        }
    }

    check_medals(ctx, 3)?;

    if labyrinth_active(ctx)? && labyrinth_result_ready(ctx)? {
        let cleared_count = get_cleared_count(ctx, AppContext::LABYRINTH)?;
        let units = labyrinth_unit_count(ctx, -1)?;

        labyrinth_submit(ctx, cleared_count, units)?;
    }

    if cleared != 0 {
        log_analytics_event(ctx, 0x64, 0, 0, 0, 0)?;
    }

    breadcrumb(ctx, 0x48)
}
