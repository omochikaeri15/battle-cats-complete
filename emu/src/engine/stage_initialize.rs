use std::{
    collections::{BTreeMap, hash_map::RandomState},
    hash::{BuildHasher, Hasher},
};

use crate::{Fault, ops};

use super::{
    AppContext, AssetStream, ENTITY_BASE, FormatArg, STAGE_DISPLAY_ORDER, aku_realm_final_redirect,
    altar_recompute, analytics_params_send, background_particles_init, base_shake_reset,
    bg_effect_spawn_all, bgm_player_bind, bgm_player_switch, breadcrumb_with,
    calculate_treasure_percentages, call_rng, cannon_start_countdown, clear_barrier_vfx,
    clear_base_guard_notice, clear_cannon_shot, clear_crit_vfx, clear_debris, clear_effect_slot,
    clear_shield_vfx, clear_wave_sprite, clear_zkill_vfx, collect_rule_id_list,
    combo_banner_pending, compute_base_health, compute_base_level, deploy_limit_reset,
    evaluate_active_combos, ex_redirect_check_a, ex_redirect_check_b, ex_redirect_check_c,
    ex_replacement_pending, fever_clear_gauge, fever_clear_state, find_fixed_lineup,
    get_background_id, get_base_max_hp, get_battle_status, get_bg_model_id,
    get_bottom_inset_logical, get_built_deck_rows, get_built_deck_stage_key, get_button_unit_id,
    get_button_unit_row, get_cannon_base_damage, get_cannon_charge_frames, get_cannon_decor_id,
    get_cannon_effect, get_cannon_foundation_id, get_cannon_id, get_cannon_part_id,
    get_cannon_part_rec, get_cannon_power, get_cannon_recharge, get_castle_enemy_row,
    get_cat_combo_bonus, get_drawable_width, get_effect_part_level, get_ex_option_target,
    get_foundation_part_id, get_global_map_id, get_item_selected, get_left_inset_logical,
    get_map_index, get_map_rules, get_map_type, get_max_money, get_max_zoom, get_powerup,
    get_powerup_available, get_right_inset_logical, get_setting, get_special_rule,
    get_special_rule_params, get_stage_index, get_stage_record, get_star_level, get_style_part_id,
    get_text_texture, get_top_inset_offset, get_unit_guide_order, get_unit_recharge,
    has_built_deck, has_castle_enemy, has_fixed_lineup, invasion_available, invasion_z_available,
    is_aku_final_map, is_ex_map_68, is_ex_option_target, is_score_stage, item_pass_active,
    labyrinth_active, labyrinth_roll_floor, load_base_models, load_battle_assets,
    load_battle_snapshot, load_lineup_preset, load_map_stage_csv, load_stage_csv,
    log_analytics_event, maanim_load, mamodel_load, mamodel_set_single_sheet, map_index_of_map_id,
    map_records_entry, min_i32, obf_value_add, open_asset_stream, option_window_init, play_sound,
    powerup_available, powerup_disabled, powerup_granted, query_localizable, read_csv_cell,
    read_csv_row, replay_mode, reset_point_state, save_battle_snapshot, scene_background_setup,
    select_point_map, set_auto_camera_mode, set_base_curse_chance, set_base_curse_duration,
    set_base_entity_frame, set_base_freeze_chance, set_base_freeze_duration, set_base_hitbox_pos,
    set_base_hitbox_width, set_base_hp, set_base_level, set_base_max_hp, set_base_occupant,
    set_base_pos_x, set_base_pos_y, set_base_slow_chance, set_base_slow_duration,
    set_base_soulstrike, set_base_state, set_battle_status, set_bgm_duck, set_cannon_base_damage,
    set_cannon_burrowed_permille, set_cannon_countdown, set_cannon_damage, set_cannon_hp_mode,
    set_cannon_makes_wave, set_cannon_metal_permille, set_cannon_nonmetal_permille,
    set_cannon_nonzombie_permille, set_cannon_parts, set_cannon_ready_vfx, set_cannon_recharge,
    set_cannon_recoil, set_cannon_shot_id, set_cannon_strike_width, set_cannon_type,
    set_cannon_unit_id, set_cannon_wall_hp_pct, set_cannon_wall_lifetime, set_cannon_wall_offset,
    set_cannon_zombie_permille, set_castle_anim_frame, set_castle_anim_state,
    set_combo_banner_pending, set_deck_cooldown, set_item_selected, set_money, set_point_stage,
    set_powerup, set_stage_unlock, set_worker_level, setup_bg_color, sound_manager,
    sound_set_channel, spawn_entity, spawn_state_init, stage_entry_row, stage_entry_start_frame,
    stage_entry_z_max, stage_entry_z_min, stage_not_sealed, std_string_from_cstr,
    string_format_int, string_format_int2, string_format_int2_text, text_texture_cache,
    validate_map_type, vibration_clear,
};

pub fn stage_initialize(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::DECK_HOLD_RELEASE, 0)?;
    ctx.set_i32_at(AppContext::DECK_HOLD_FRAMES, 0)?;
    ctx.set_i32_at(AppContext::DECK_HOLD_SLOT, -1)?;
    ctx.set_block_at::<1>(AppContext::DECK_BUTTON_HELD, [0])?;
    base_shake_reset(&mut ctx.base_shake);
    bgm_player_bind(ctx);
    ctx.set_i32_at(AppContext::TUTORIAL_STEP, 0)?;
    vibration_clear(ctx);

    let ex_map = if ex_redirect_check_a(ctx)? {
        Some(0x1a)
    } else if ex_redirect_check_b(ctx)? {
        Some(0x45)
    } else if ex_redirect_check_c(ctx, 1)? {
        Some(0x44)
    } else if aku_realm_final_redirect(ctx, 1)? {
        Some(0x2a)
    } else {
        None
    };

    if let Some(ex_map) = ex_map {
        ctx.set_i32_at(
            AppContext::OUTRO_CHAPTER_MODE,
            ctx.i32_at(AppContext::CHAPTER_MODE)?,
        )?;
        ctx.set_i32_at(
            AppContext::OUTRO_ENTRY_STAGE,
            ctx.i32_at(AppContext::ENTRY_STAGE)?,
        )?;
        ctx.set_i32_at(AppContext::EX_MAP, ex_map)?;
        ctx.set_i32_at(AppContext::EX_STAGE, 0)?;
        ctx.set_i32_at(AppContext::CHAPTER_MODE, 0x63)?;
        ctx.set_i32_at(AppContext::ENTRY_STAGE, 0)?;
    } else if ex_replacement_pending(ctx, -1, -1)? {
        ctx.set_i32_at(
            AppContext::OUTRO_CHAPTER_MODE,
            ctx.i32_at(AppContext::CHAPTER_MODE)?,
        )?;
        ctx.set_i32_at(
            AppContext::OUTRO_ENTRY_STAGE,
            ctx.i32_at(AppContext::ENTRY_STAGE)?,
        )?;

        let map_id = get_global_map_id(ctx, 0)?;
        let target = get_ex_option_target(ctx, map_id);

        ctx.set_i32_at(AppContext::EX_MAP, map_index_of_map_id(target))?;

        let stage = get_stage_index(ctx)?;

        ctx.set_i32_at(AppContext::EX_STAGE, stage)?;
        ctx.set_i32_at(AppContext::CHAPTER_MODE, 0x63)?;
        ctx.set_i32_at(AppContext::ENTRY_STAGE, stage)?;
    } else if labyrinth_active(ctx)? {
        labyrinth_roll_floor(ctx, 1)?;
    }

    let map_id = get_global_map_id(ctx, 0)?;

    if map_records_entry(ctx, map_id) {
        let map_id = get_global_map_id(ctx, 0)?;

        if !ctx.condition_list_200k.contains(&map_id) {
            let map_id = get_global_map_id(ctx, 0)?;

            ctx.condition_list_200k.push(map_id);
        }
    }

    ctx.set_i32_at(AppContext::LOST_MAP_TYPE, -1)?;
    ctx.set_i32_at(AppContext::LOST_MAP_TYPE + 4, -1)?;
    ctx.set_i32_at(AppContext::LOST_MAP_TYPE + 8, -1)?;
    ctx.item_drop_queue.clear();
    ctx.item_snapshot.clear();
    ctx.attackers_by_serial[0].clear();

    if ctx.u8_at(AppContext::SCORE_MODE_FLAG)? != 0 {
        ctx.set_block_at::<8>(AppContext::SCORE_TOTAL, [0; 8])?;
    }

    if get_map_type(ctx, 0)? == -11 {
        let stage = get_stage_index(ctx)?;
        let group = ops::div_100(
            ctx.i32_at(AppContext::DROP_MAP_STAGES + stage as i64 as usize * 4)?,
        );
        let name = string_format_int(ctx, b"MapStageDataN_%03d.csv", group)?;

        if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
            let mut stm = AssetStream::new(&bytes, b'\n');

            read_csv_row(&mut stm);
            read_csv_row(&mut stm);

            let mut row = 0i32;

            loop {
                let stage = get_stage_index(ctx)?;
                let packed = ctx.i32_at(AppContext::DROP_MAP_STAGES + stage as i64 as usize * 4)?;

                if row > packed.wrapping_sub(ops::div_100(packed).wrapping_mul(100)) {
                    break;
                }

                read_csv_row(&mut stm);
                row += 1;
            }

            for col in 2..5usize {
                let stage = get_stage_index(ctx)?;
                let value = read_csv_cell(&stm, col as i32) as i32;
                let row_at = AppContext::MAP_STAGE_ROWS
                    .wrapping_add((stage as i64 as usize).wrapping_mul(0xbc));
                let key = ctx.i32_at(row_at + 0xb8)?;

                ctx.set_i32_at(row_at + col * 4, value ^ key)?;
            }
        }
    }

    deploy_limit_reset(ctx)?;

    if ctx.i32_at(AppContext::BATTLE_RESUMED)? != 0 {
        load_battle_snapshot(ctx, 0)?;

        if ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0 {
            let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;
            let (saga, map) = if mode < 3 {
                (0, mode)
            } else {
                (
                    (mode >= 7) as i32 + 1,
                    (mode - 1) - ((mode - 1) as u32 / 3 * 3) as i32,
                )
            };
            let name = string_format_int2(ctx, b"stageNormal%d_%d_Z.csv", saga, map)?;

            if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
                let mut stm = AssetStream::new(&bytes, b'\n');

                read_csv_row(&mut stm);
                read_csv_row(&mut stm);

                let mut row = 0usize;

                loop {
                    read_csv_row(&mut stm);

                    if (row as i32) < ctx.i32_at(AppContext::ENTRY_STAGE)? {
                        if row > 0x62 {
                            break;
                        }

                        row += 1;
                        continue;
                    }

                    let row_at = AppContext::MAP_STAGE_ROWS + row * 0xbc;

                    for col in 0..0x2eusize {
                        let value = read_csv_cell(&stm, col as i32) as i32;
                        let key = ctx.i32_at(row_at + 0xb8)?;

                        ctx.set_i32_at(row_at + col * 4, value ^ key)?;

                        if ctx.i32_at(row_at + col * 4)? ^ ctx.i32_at(row_at + 0xb8)? == -1 {
                            break;
                        }
                    }

                    if ctx.i32_at(row_at)? ^ ctx.i32_at(row_at + 0xb8)? == -1 || row >= 0x63 {
                        break;
                    }

                    row += 1;
                }
            }
        }

        let invasion = ctx.u8_at(AppContext::BATTLE_IS_INVASION)?;

        if invasion != 0 || ctx.u8_at(AppContext::BATTLE_IS_Z_INVASION)? != 0 {
            let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;
            let (saga, map) = if mode < 3 {
                (0, mode)
            } else {
                (
                    (mode >= 7) as i32 + 1,
                    (mode - 1) - ((mode - 1) as u32 / 3 * 3) as i32,
                )
            };
            let suffix: &[u8] = if invasion == 0 { b"_Z" } else { b"" };
            let name = string_format_int2_text(
                ctx,
                b"stageNormal%d_%d_Invasion%s.csv",
                saga,
                map,
                suffix,
            )?;

            if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
                let mut stm = AssetStream::new(&bytes, b'\n');

                read_csv_row(&mut stm);
                read_csv_row(&mut stm);
                read_csv_row(&mut stm);

                if ctx.u8_at(AppContext::INVASION_STAGE)? <= 0x63 {
                    for col in 0..0x2eusize {
                        let value = read_csv_cell(&stm, col as i32) as i32;
                        let row = ctx.u8_at(AppContext::INVASION_STAGE)? as i8 as isize as usize;
                        let row_at =
                            AppContext::MAP_STAGE_ROWS.wrapping_add(row.wrapping_mul(0xbc));
                        let key = ctx.i32_at(row_at + 0xb8)?;

                        ctx.set_i32_at(row_at + col * 4, value ^ key)?;

                        let row = ctx.u8_at(AppContext::INVASION_STAGE)? as i8 as isize as usize;
                        let row_at =
                            AppContext::MAP_STAGE_ROWS.wrapping_add(row.wrapping_mul(0xbc));

                        if ctx.i32_at(row_at + col * 4)? ^ ctx.i32_at(row_at + 0xb8)? == -1 {
                            break;
                        }
                    }
                }
            }
        }
    }

    if ctx.i32_at(AppContext::BATTLE_RESUMED)? == 0 {
        ctx.set_block_at::<1>(AppContext::BATTLE_IS_OUTBREAK, [0])?;

        if ctx.u8_at(AppContext::OUTBREAKS_ENABLED)? != 0 {
            let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;
            let stage = ctx.i32_at(AppContext::ENTRY_STAGE)?;
            let active = *ctx
                .outbreak_active
                .entry(mode)
                .or_default()
                .entry(stage)
                .or_default();

            if active {
                ctx.set_block_at::<1>(AppContext::BATTLE_IS_OUTBREAK, [1])?;
            }
        }

        ctx.set_block_at::<1>(AppContext::BATTLE_IS_INVASION, [0])?;

        let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;

        if invasion_available(ctx, mode)?
            && ctx.i32_at(AppContext::ENTRY_STAGE)?
                == ctx.u8_at(AppContext::INVASION_STAGE)? as i8 as i32
        {
            ctx.set_block_at::<1>(AppContext::BATTLE_IS_INVASION, [1])?;
        }

        ctx.set_block_at::<1>(AppContext::BATTLE_IS_Z_INVASION, [0])?;

        let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;

        if invasion_z_available(ctx, mode)?
            && ctx.i32_at(AppContext::ENTRY_STAGE)?
                == ctx.u8_at(AppContext::INVASION_STAGE)? as i8 as i32
        {
            ctx.set_block_at::<1>(AppContext::BATTLE_IS_Z_INVASION, [1])?;
        }

        log_analytics_event(ctx, 0x2e, 0, 0, 0, 0)?;
        ctx.set_block_at::<1>(AppContext::MISSION_CANNON_FIRED, [0])?;
    }

    ctx.set_i32_at(AppContext::EX_MAP_INDEX, ctx.i32_at(AppContext::EX_MAP)?)?;
    ctx.set_i32_at(
        AppContext::EX_STAGE_INDEX,
        ctx.i32_at(AppContext::EX_STAGE)?,
    )?;

    if ctx.i32_at(AppContext::BATTLE_RESUMED)? != 0 {
        let amount = ctx.i32_at(AppContext::RESUMED_MONEY)?;

        set_money(ctx, AppContext::faction_flags(0), amount)?;
    }

    let map_id = get_global_map_id(ctx, 0)?;
    let map_text = string_format_int(ctx, b"%d", map_id)?;
    let star = get_star_level(ctx)?;
    let star_text = string_format_int(ctx, b"%d", star)?;
    let stage = get_stage_index(ctx)?;
    let stage_text = string_format_int(ctx, b"%d", stage)?;
    let adoption: &[u8] = if ctx.u8_at(AppContext::USE_BUILT_DECK)? != 0 {
        b"true"
    } else {
        b"false"
    };
    let mut params: Vec<(Vec<u8>, Vec<u8>)> = vec![
        (b"MapID".to_vec(), map_text),
        (b"Level".to_vec(), star_text),
        (b"StageIndex".to_vec(), stage_text),
        (b"useClearedAdoption".to_vec(), adoption.to_vec()),
    ];

    for slot in 0..10usize {
        let preset = ctx.i32_at(AppContext::SELECTED_DECK_PRESET)? as i64 as usize;
        let row =
            ctx.bytes_from(AppContext::DECK_PRESETS.wrapping_add(preset.wrapping_mul(0x2c)))?;
        let unit = ops::xor_row_decode(row, 10, slot).ok_or(Fault::index_out_of_range(slot as i64, 10))? as i32;
        let key = string_format_int(ctx, b"Unit%d", slot as i32)?;
        let value = if unit > 0 {
            let form = ctx.i32_at(AppContext::UNIT_FORMS + unit as u32 as usize * 4)?;

            string_format_int2(ctx, b"%03d,%d", unit, form)?
        } else {
            b"None".to_vec()
        };

        if !params.iter().any(|(name, _)| *name == key) {
            params.push((key, value));
        }
    }

    params.sort();

    let pairs: Vec<(&[u8], &[u8])> = params
        .iter()
        .map(|(name, value)| (name.as_slice(), value.as_slice()))
        .collect();

    breadcrumb_with(ctx, 0x45, &pairs)?;

    if let Some(preset) = find_fixed_lineup(ctx, -1, -1, -1)? {
        load_lineup_preset(ctx, preset.as_bytes())?;

        let key = ctx.i32_at(AppContext::BATTLE_DECK_KEY)?;

        for slot in 0..10usize {
            ctx.set_i32_at(AppContext::BATTLE_DECK + slot * 4, !key)?;
        }

        let units: Vec<i32> = ctx
            .fixed_lineup_store
            .units
            .iter()
            .map(|unit| unit.unit_id)
            .collect();

        for (slot, unit) in units.iter().enumerate() {
            let key = ctx.i32_at(AppContext::BATTLE_DECK_KEY)?;

            ctx.set_i32_at(
                AppContext::BATTLE_DECK + slot * 4,
                unit.wrapping_add(2) ^ key,
            )?;
        }
    } else {
        let stage_key = get_built_deck_stage_key(ctx)?;

        ctx.set_i32_at(AppContext::BUILT_DECK_EX_STAGE_KEY, stage_key)?;

        let stage_key = get_built_deck_stage_key(ctx)?;
        let rows = get_built_deck_rows(ctx, stage_key)?;
        let mut count = 0i32;

        for (slot, row) in rows.iter().enumerate() {
            let previous = count;

            if ctx.i32_at(AppContext::BATTLE_RESUMED)? == 0 {
                let built = ctx.u8_at(AppContext::USE_BUILT_DECK)? != 0 && {
                    let stage_key = get_built_deck_stage_key(ctx)?;

                    has_built_deck(ctx, stage_key)?
                };
                let unit = if built {
                    row.0 as i32
                } else {
                    let preset = ctx.i32_at(AppContext::SELECTED_DECK_PRESET)? as i64 as usize;
                    let row = ctx.bytes_from(
                        AppContext::DECK_PRESETS.wrapping_add(preset.wrapping_mul(0x2c)),
                    )?;

                    ops::xor_row_decode(row, 10, slot).ok_or(Fault::index_out_of_range(slot as i64, 10))? as i32
                };
                let key = ctx.i32_at(AppContext::BATTLE_DECK_KEY)?;

                ctx.set_i32_at(AppContext::BATTLE_DECK + slot * 4, unit ^ key)?;
            }

            count = previous.wrapping_add((get_button_unit_id(ctx, 0, slot as i32)? >= 0) as i32);
        }

        let deck_count = count;

        if get_map_type(ctx, 0)? != -11 {
            ctx.set_i32_at(AppContext::DEPLOY_FULL_FLASH, -1)?;
        } else {
            let last = (ctx.random_dungeon_rows.len() as u32).wrapping_sub(1) as i32;
            let map_index = get_map_index(ctx, 0)?;
            let stage = get_stage_index(ctx)?;
            let star = get_star_level(ctx)?;
            let clears = *ctx
                .dungeon_clear_counts
                .get(map_index as i64 as usize)
                .and_then(|map| map.get(stage as i64 as usize))
                .and_then(|stage| stage.get(star as i64 as usize))
                .ok_or(Fault::index_out_of_range(map_index as i64, ctx.dungeon_clear_counts.len() as i64))?;
            let pick = min_i32(last, clears as i32);
            let weights = *ctx.random_dungeon_rows.get(pick as i64 as usize).ok_or(
                Fault::index_out_of_range(pick as i64, ctx.random_dungeon_rows.len() as i64),
            )?;
            let total = weights[..8]
                .iter()
                .fold(0i32, |sum, weight| sum.wrapping_add(*weight))
                .wrapping_add(weights[8])
                .wrapping_add(weights[9].wrapping_add(weights[10]));
            let roll = call_rng(ctx, total);
            let mut bound = weights[0];
            let mut locks = 10;

            for (index, weight) in weights.iter().enumerate().skip(1).take(9) {
                if roll < bound {
                    break;
                }

                bound = bound.wrapping_add(*weight);
                locks = if index == 9 {
                    (roll < bound) as i32
                } else {
                    10 - index as i32
                };
            }

            let chosen = locks;
            let locked = if deck_count < chosen {
                deck_count
            } else {
                chosen
            };
            let mut picks: BTreeMap<i32, bool> = BTreeMap::new();

            for _ in 0..locked.max(0) {
                let mut slot = call_rng(ctx, deck_count);

                if !*picks.entry(slot).or_default() {
                    *picks.entry(slot).or_default() = true;
                    continue;
                }

                let mut tries = 0;

                loop {
                    slot = ops::irem(slot.wrapping_add(1), deck_count)
                        .ok_or(Fault::divide(deck_count as i64))?;

                    if !*picks.entry(slot).or_default() {
                        *picks.entry(slot).or_default() = true;
                        break;
                    }

                    tries += 1;

                    if tries == locked {
                        break;
                    }
                }
            }

            let mut owned: Vec<i32> = Vec::new();

            for unit in 0..0x36c {
                if get_unit_guide_order(ctx, unit)? == -1 {
                    continue;
                }

                let mut pair = [0u8; 8];

                pair[..4].copy_from_slice(
                    &ctx.block_at::<4>(AppContext::UNITS_OWNED + unit as usize * 4)?,
                );
                pair[4..].copy_from_slice(&ctx.block_at::<4>(AppContext::UNITS_OWNED_KEY)?);

                if ops::xor_row_decode(&pair, 1, 0).ok_or(Fault::index_out_of_range(0, 1))? == 0
                {
                    continue;
                }

                owned.push(unit);
            }

            if deck_count != 0 {
                for slot in 0..deck_count {
                    let unit = get_button_unit_id(ctx, 0, slot)?;

                    if let Some(found) = owned.iter().position(|candidate| *candidate == unit) {
                        owned.remove(found);
                    }
                }
            }

            let mut index = 0usize;

            while index + 1 < owned.len() {
                let span = (owned.len() - 1 - index) as u64;
                let offset = (RandomState::new().build_hasher().finish() % (span + 1)) as usize;

                if offset != 0 {
                    owned.swap(index, index + offset);
                }

                index += 1;
            }

            let mut taken = 0usize;

            for slot in 0..deck_count.max(0) {
                if taken >= owned.len() {
                    break;
                }

                if *picks.entry(slot).or_default() {
                    let key = ctx.i32_at(AppContext::BATTLE_DECK_KEY)?;

                    ctx.set_i32_at(
                        AppContext::BATTLE_DECK + slot as usize * 4,
                        owned[taken].wrapping_add(2) ^ key,
                    )?;
                    taken += 1;
                }
            }

            let map_index = get_map_index(ctx, 0)?;
            let stage = get_stage_index(ctx)?;
            let star = get_star_level(ctx)?;
            let cell = ctx
                .dungeon_clear_counts
                .get_mut(map_index as i64 as usize)
                .and_then(|map| map.get_mut(stage as i64 as usize))
                .and_then(|stage| stage.get_mut(star as i64 as usize))
                .ok_or(Fault::index_out_of_range(map_index as i64, 0))?;

            if *cell <= 0x270e {
                *cell = cell.wrapping_add(1);
            }

            ctx.set_i32_at(
                AppContext::DEPLOY_FULL_FLASH,
                if chosen == 0 { -1 } else { 0 },
            )?;
        }
    }

    evaluate_active_combos(ctx)?;
    ctx.set_block_at::<0x14>(AppContext::COMBO_BANNER_UNITS, [0xff; 0x14])?;
    ctx.set_i32_at(
        AppContext::COMBO_BANNER_STATE,
        if ctx.i32_at(AppContext::DEPLOY_FULL_FLASH)? == -1 {
            0
        } else {
            -0xf
        },
    )?;
    ctx.set_block_at::<0x10>(AppContext::COMBO_BANNER_PHASE, [0; 0x10])?;

    let map_id = get_global_map_id(ctx, 0)?;
    let banned = collect_rule_id_list(&mut ctx.special_rules, map_id);

    if !banned.is_empty() {
        for record in ctx.combo_store.records.iter_mut() {
            if record.enabled == 0 || record.effect_count <= 0 {
                continue;
            }

            for index in 0..record.effect_count as usize {
                let effects = [
                    record.kind[0],
                    record.kind[1],
                    record.kind[2],
                    record.power[0],
                    record.power[1],
                    record.power[2],
                    record.effect_count,
                ];
                let kind = *effects.get(index).ok_or(Fault::index_out_of_range(index as i64, 7))?;

                if banned.contains(&kind) {
                    record.banner_pending = 0;
                    record.enabled = 0;
                    break;
                }
            }
        }
    }

    if ctx.i32_at(AppContext::BATTLE_RESUMED)? != 0 {
        for record in ctx.combo_store.records.iter_mut() {
            if combo_banner_pending(record) != 0 {
                set_combo_banner_pending(record, 0);
            }
        }
    }

    ctx.set_i32_at(AppContext::OUTRO_TICKS, 0)?;
    ctx.set_i32_at(AppContext::RANK_REWARD_BASE, 0)?;
    sound_set_channel(sound_manager(ctx)?, 5, 5);
    ctx.set_block_at::<1>(AppContext::RANK_POPUP_SHOWN, [0])?;
    ctx.set_block_at::<1>(AppContext::BASE_KILL_BLOCKED, [0])?;

    let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;

    ctx.set_i32_at(
        AppContext::CHAPTER_COST_TIER,
        if mode < 3 { mode } else { 1 },
    )?;
    ctx.set_i32_at(AppContext::MAP_RETURN_FLAG, 0)?;
    ctx.set_block_at::<0x28>(AppContext::DRAW_TEMP_0, [0; 0x28])?;

    let resumed = ctx.i32_at(AppContext::BATTLE_RESUMED)? != 0;

    if resumed {
        fever_clear_state(&mut ctx.special_rules);
    } else {
        fever_clear_gauge(&mut ctx.special_rules);

        let mut slot = AppContext::WAVE_RECORDS;

        while slot != AppContext::WAVE_SPRITES {
            clear_effect_slot(ctx, slot)?;
            slot += 0x30;
        }

        let mut block = AppContext::WAVE_SPRITES;

        while block != 0x33886c {
            for part in 0..6usize {
                clear_wave_sprite(ctx, block + part * 8)?;
            }

            block += 0x30;
        }

        ctx.set_block_at::<0x27d8>(AppContext::WAVE_HITS, [0; 0x27d8])?;
        ctx.surge_events.clear();
        ctx.counter_surge_events.clear();
        ctx.explosion_events.clear();
        ctx.set_i32_at(AppContext::BATTLE_INTRO_FRAME, 0)?;
        ctx.set_i32_at(AppContext::KILLS_SINCE_SPAWN_TICK, 0)?;
        ctx.set_block_at::<1>(AppContext::DRAG_LATCHED, [0])?;
        ctx.set_block_at::<0x14>(AppContext::BGM_SWITCH_STATE, [0; 0x14])?;
        ctx.set_block_at::<1>(AppContext::SPEED_UP_LATCH, [0])?;

        for item in 0..6 {
            if get_item_selected(ctx, item)? && powerup_disabled(ctx, item)? {
                set_item_selected(ctx, item, 0)?;
            }
        }

        for powerup in 0..6i32 {
            if !get_item_selected(ctx, powerup)? {
                set_powerup(ctx, powerup, 0)?;
                continue;
            }

            let granted = ctx.u8_at(AppContext::SCORE_MODE_FLAG)? != 0
                || item_pass_active(ctx, powerup)?
                || powerup_granted(ctx, powerup)?;
            let consume = !granted || (powerup == 0 && get_powerup_available(ctx)? != 0);

            if consume {
                ctx.set_block_at::<1>(AppContext::POWERUP_FREE + powerup as usize, [0])?;

                let spend = ctx.i32_at(AppContext::CHAPTER_MODE)? != 0x63
                    || ex_redirect_check_a(ctx)?
                    || ex_redirect_check_b(ctx)?
                    || is_ex_map_68(ctx)?
                    || is_aku_final_map(ctx)?
                    || is_ex_option_target(ctx)?;

                if spend {
                    let at = AppContext::ITEM_COUNTS_KIND_3 + powerup as usize * 8;
                    let mut cell = ctx.block_at::<8>(at)?;

                    obf_value_add(&mut cell, -1);
                    ctx.set_block_at(at, cell)?;
                }
            } else {
                ctx.set_block_at::<1>(AppContext::POWERUP_FREE + powerup as usize, [1])?;
            }

            set_powerup(ctx, powerup, 1)?;

            if powerup == 0 && get_powerup_available(ctx)? != 0 {
                ctx.set_block_at::<1>(AppContext::SPEED_UP_LATCH, [1])?;
            }
        }

        let stage_key = get_built_deck_stage_key(ctx)?;
        let rows = get_built_deck_rows(ctx, stage_key)?;

        if has_fixed_lineup(ctx, -1, -1, -1)? {
            let forms: Vec<i32> = ctx
                .fixed_lineup_store
                .units
                .iter()
                .map(|unit| unit.form)
                .collect();

            for (slot, form) in forms.iter().enumerate() {
                ctx.set_i32_at(AppContext::BUTTON_UNIT_FORMS + slot * 4, *form)?;
            }
        } else if ctx.i32_at(AppContext::BATTLE_RESUMED)? == 0 {
            for slot in 0..10i32 {
                let form = if get_button_unit_id(ctx, 0, slot)? < 0 {
                    0
                } else if ctx.u8_at(AppContext::USE_BUILT_DECK)? != 0 && {
                    let stage_key = get_built_deck_stage_key(ctx)?;

                    has_built_deck(ctx, stage_key)?
                } {
                    rows[slot as usize].1 as i32
                } else {
                    let unit = get_button_unit_id(ctx, 0, slot)?;

                    ctx.i32_at(
                        AppContext::UNIT_FORMS.wrapping_add((unit as i64 as usize).wrapping_mul(4)),
                    )?
                };

                ctx.set_i32_at(AppContext::BUTTON_UNIT_FORMS + slot as usize * 4, form)?;
            }
        }

        let mut filled = 0;

        for slot in 0..10 {
            if get_button_unit_row(ctx, 0, slot)? != -1 {
                filled += 1;
            }
        }

        ctx.set_block_at::<1>(AppContext::DECK_BACK_ROW_ENABLED, [(filled >= 6) as u8])?;
        ctx.set_block_at::<2>(AppContext::OPTION_MENU_IS_OPEN, [0; 2])?;
        ctx.set_block_at::<2>(AppContext::CAT_GOD_MENU_IS_OPEN, [0; 2])?;
        ctx.set_block_at::<0x28>(AppContext::CAT_GOD_BUTTON_PRESS, [0; 0x28])?;
        ctx.set_block_at::<0x13>(AppContext::CANNON_HELD, [0; 0x13])?;
        ctx.set_i32_at(AppContext::CANNON_HELD + 0xf, 0)?;
        ctx.set_block_at::<0x8c>(AppContext::STAGE_ROW, [0; 0x8c])?;
        ctx.set_block_at::<0x20>(AppContext::STAGE_LENGTH, [0; 0x20])?;
        ctx.set_i32_at(AppContext::STAGE_BOSS_GUARD, 0)?;
        ctx.stage_enemies.clear();
        ctx.set_i32_at(AppContext::SPAWN_COUNTDOWN, 0)?;
        ctx.set_block_at::<0x10>(AppContext::BLINK_COUNTER, [0; 0x10])?;
        calculate_treasure_percentages(ctx)?;

        if has_fixed_lineup(ctx, -1, -1, -1)? {
            let flags: Vec<(i32, bool)> = ctx
                .fixed_lineup_store
                .treasure_flags
                .iter()
                .map(|(chapter, flag)| (*chapter, *flag))
                .collect();

            for (chapter, flag) in flags {
                let value = if flag { 100 } else { 0 };

                for group in 0..0xbusize {
                    if chapter == 3 {
                        continue;
                    }

                    let groups = ctx.treasure_store.get(chapter as i64 as usize).ok_or(
                        Fault::index_out_of_range(chapter as i64, 10),
                    )?;
                    let effect = groups
                        .get(group)
                        .ok_or(Fault::index_out_of_range(group as i64, groups.len() as i64))?
                        .effect;

                    if (effect.wrapping_sub(7) as u32) < 2 {
                        continue;
                    }

                    ctx.set_i32_at(
                        AppContext::TREASURE_PROGRESS
                            .wrapping_add((chapter as i64 as usize).wrapping_mul(0x2c))
                            .wrapping_add(group * 4),
                        value,
                    )?;
                }
            }
        }

        ctx.set_i32_at(AppContext::REVIVE_REQUESTED, 0)?;
        ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
        ctx.set_block_at::<0x2c>(AppContext::SPEED_UP_PRESS, [0; 0x2c])?;
        ctx.set_i32_at(AppContext::CAT_GOD_GLOW_TIMER, 0)?;
        ctx.set_block_at::<8>(AppContext::DECK_BAR_SLIDE, [0; 8])?;
        ctx.set_block_at::<0x1ac>(AppContext::CANNON_RECT, [0; 0x1ac])?;

        let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;

        if mode == 3 {
            let map_type = validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?);
            let map_index = ctx.i32_at(AppContext::MAP_INDEX)?;
            let star = ctx.i32_at(AppContext::STAR_LEVEL)?;
            let stage = ctx.i32_at(AppContext::ENTRY_STAGE)?;
            let replay = replay_mode(ctx)? as i32;

            set_stage_unlock(ctx, map_type, map_index, star, stage, replay)?;
        } else if mode == 0x63 {
            if is_aku_final_map(ctx)? {
                set_stage_unlock(ctx, -0x13, 0, 0, 0x1d, 0)?;
            }
        } else {
            let stage = ctx.i32_at(AppContext::ENTRY_STAGE)?;

            if stage.wrapping_sub(0x30) as u32 >= 3 {
                ctx.set_i32_at(
                    AppContext::STAGE_UNLOCK_CHAPTERS
                        .wrapping_add((mode as i64 as usize).wrapping_mul(4)),
                    stage,
                )?;
            }
        }

        ctx.set_i32_at(AppContext::STAGE_ROW, ctx.i32_at(AppContext::ENTRY_STAGE)?)?;
        ctx.set_block_at::<0x14>(AppContext::STAGE_NO_CONTINUES, [0; 0x14])?;

        let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;
        let stage = ctx.i32_at(AppContext::ENTRY_STAGE)?;
        let shift = if mode == 0x63 || mode == 3 {
            0
        } else if mode == 1 && stage == 0x2f {
            2
        } else if mode == 2 && stage == 0x2f {
            3
        } else {
            0
        };

        if !load_stage_csv(ctx, stage.wrapping_add(shift), 1)? {
            return Ok(());
        }

        let background = get_background_id(ctx)?;

        setup_bg_color(ctx, background)?;

        let row = ctx.i32_at(AppContext::STAGE_ROW)?;
        let display = if ctx.i32_at(AppContext::CHAPTER_MODE)? == 3 {
            row
        } else {
            *STAGE_DISPLAY_ORDER
                .get(row as i64 as usize)
                .ok_or(Fault::index_out_of_range(row as i64, 0x33))?
        };

        ctx.set_i32_at(AppContext::CASTLE_ID, display)?;

        let length = ctx.i32_at(AppContext::STAGE_LENGTH)?;

        ctx.set_i32_at(AppContext::CAMERA_X, length.wrapping_sub(0x2580))?;

        if ctx.i32_at(AppContext::TUTORIAL_CLEARED)? == 0 {
            let zoom = get_max_zoom(ctx)? as f32 / 100.0 / 100.0;
            let length = ctx.i32_at(AppContext::STAGE_LENGTH)?;

            ctx.set_i32_at(
                AppContext::CAMERA_X,
                ops::cvttss2si((-9600.0f32 / zoom + length as f32) * 0.5),
            )?;
        }

        ctx.set_i32_at(AppContext::OUTRO_PHASE, 0)?;

        let length = ctx.i32_at(AppContext::STAGE_LENGTH)?;

        ctx.set_i32_at(
            AppContext::CAMERA_MIN_ZOOM,
            ops::idiv(0xea600, length)
                .ok_or(Fault::divide(length as i64))?
                .wrapping_add(1),
        )?;

        let zoom = get_max_zoom(ctx)?;

        ctx.set_i32_at(AppContext::CAMERA_ZOOM, zoom)?;

        let map_id = get_global_map_id(ctx, 0)?;

        select_point_map(ctx, map_id)?;

        if is_score_stage(ctx.event_items.as_ref()) {
            reset_point_state(ctx)?;

            let stage = get_stage_index(ctx)?;

            set_point_stage(ctx, stage)?;
        }

        set_battle_status(ctx, 3)?;

        let wallet = AppContext::faction_flags(0);

        for slot in 0..10i32 {
            let map_id = get_global_map_id(ctx, 0)?;

            if !get_special_rule(ctx, &ctx.special_rules, map_id, 0)? {
                set_deck_cooldown(ctx, wallet, slot, 0, 1)?;
            } else {
                let map_id = get_global_map_id(ctx, 0)?;
                let item = get_map_rules(&ctx.special_rules, map_id)?
                    .ok_or(Fault::null_pointer())?
                    .contents_type;

                if item == 2 {
                    let recharge = get_unit_recharge(ctx, 0, slot)?;
                    let cut = get_setting(&ctx.settings, b"battle_sentai_recast", 0x32)?;
                    let value =
                        ops::div_100(100i32.wrapping_sub(cut).wrapping_mul(recharge) as i64)
                            as i32;

                    set_deck_cooldown(ctx, wallet, slot, value, 1)?;
                } else {
                    let map_id = get_global_map_id(ctx, 0)?;
                    let params = get_special_rule_params(ctx, &ctx.special_rules, map_id, 0)?
                        .cloned()
                        .unwrap_or_default();
                    let recharge = get_unit_recharge(ctx, 0, slot)?;

                    if params.len() * 4 < 5 {
                        set_deck_cooldown(ctx, wallet, slot, recharge, 1)?;
                    } else {
                        let cut = *params.get(1).ok_or(Fault::index_out_of_range(1, params.len() as i64))?;
                        let value = ops::div_100(
                            100i32.wrapping_sub(cut).wrapping_mul(recharge) as i64,
                        ) as i32;

                        set_deck_cooldown(ctx, wallet, slot, value, 1)?;
                    }
                }
            }

            ctx.set_i32_at(
                wallet + AppContext::WALLET_RED_GAUGE_FRAMES + slot as usize * 4,
                0,
            )?;
            ctx.set_i32_at(
                wallet + AppContext::WALLET_CONJURE_READY + slot as usize * 4,
                0,
            )?;
            ctx.set_i32_at(
                wallet + AppContext::WALLET_DEPLOY_COUNTS + slot as usize * 4,
                0,
            )?;
            ctx.set_i32_at(
                wallet + AppContext::WALLET_ORB_DEPLOYS_SEEN + slot as usize * 4,
                0,
            )?;
            ctx.set_i32_at(
                wallet + AppContext::WALLET_ESCALATING_COSTS + slot as usize * 4,
                0,
            )?;
            ctx.set_i32_at(
                wallet + AppContext::WALLET_SLOT_FLASH + slot as usize * 4,
                -1,
            )?;
            ctx.set_block_at::<1>(wallet + AppContext::WALLET_CANNON_FIRED, [0])?;
        }

        let worker = if get_powerup(ctx, 2)? {
            7
        } else {
            let map_id = get_global_map_id(ctx, 0)?;

            if get_special_rule(ctx, &ctx.special_rules, map_id, 0)? {
                7
            } else if get_cat_combo_bonus(ctx, &ctx.combo_store, 4, -1)? > 0 {
                get_cat_combo_bonus(ctx, &ctx.combo_store, 4, -1)?
            } else {
                0
            }
        };

        set_worker_level(ctx, wallet, worker)?;

        let money = get_cat_combo_bonus(ctx, &ctx.combo_store, 5, -1)?.wrapping_mul(100);

        set_money(ctx, wallet, money)?;

        let map_id = get_global_map_id(ctx, 0)?;

        if get_special_rule(ctx, &ctx.special_rules, map_id, 0)? {
            let max = get_max_money(ctx, wallet)?;

            set_money(ctx, wallet, max)?;
        }

        ctx.deploy_queue.clear();

        let enemy_wallet = AppContext::faction_flags(1);

        for slot in 0..10i32 {
            set_deck_cooldown(ctx, enemy_wallet, slot, 0, 1)?;
            ctx.set_i32_at(
                enemy_wallet + AppContext::WALLET_CONJURE_READY + slot as usize * 4,
                0,
            )?;
        }

        set_worker_level(ctx, enemy_wallet, 0)?;
        set_money(ctx, enemy_wallet, 0)?;
        ctx.set_i32_at(wallet + AppContext::WALLET_SPAWN_SERIAL, 0)?;
        ctx.attackers_by_serial[0].clear();
        ctx.set_i32_at(enemy_wallet + AppContext::WALLET_SPAWN_SERIAL, 0)?;
        ctx.attackers_by_serial[1].clear();
        ctx.set_block_at::<0x18e70>(ENTITY_BASE, [0; 0x18e70])?;
        set_base_occupant(ctx, 0)?;
        set_base_state(ctx, 0, 0)?;
        set_base_entity_frame(ctx, 0, 0)?;

        let length = ctx.i32_at(AppContext::STAGE_LENGTH)?;

        set_base_pos_x(ctx, 0, length.wrapping_add(-0xc80))?;
        set_base_pos_y(ctx, 0, 0x1130)?;

        let health = compute_base_health(ctx)?;

        set_base_max_hp(ctx, 0, health)?;

        let health = get_base_max_hp(ctx, 0)?;

        set_base_hp(ctx, 0, health)?;
        set_base_hitbox_pos(ctx, 0, 0)?;
        set_base_hitbox_width(ctx, 0, 0x708)?;

        let level = compute_base_level(ctx)?;

        set_base_level(ctx, 0, level)?;
        set_castle_anim_state(ctx, 0, 0)?;
        set_castle_anim_frame(ctx, 0, 0)?;

        let charge = get_cannon_charge_frames(ctx)?;

        set_cannon_recharge(ctx, 0, charge)?;

        let recharge = get_cannon_recharge(ctx, 0)?;
        let countdown = cannon_start_countdown(ctx, recharge)?;

        set_cannon_countdown(ctx, 0, countdown)?;
        set_cannon_shot_id(ctx, 0, 0)?;

        let power = get_cannon_power(ctx)?;

        set_cannon_base_damage(ctx, 0, power)?;

        let part = get_cannon_part_id(ctx)?;
        let saved = get_effect_part_level(ctx, part)?;
        let fixed = has_fixed_lineup(ctx, -1, -1, -1)?;
        let preset = ctx.i32_at(AppContext::LINEUP_CANNON_LEVEL)?;
        let level = if preset == -1 || !fixed {
            saved
        } else {
            preset
        }
        .wrapping_add(1);
        let cannon_id = get_cannon_part_id(ctx)?;
        let style_id = get_style_part_id(ctx)?;
        let foundation_id = get_foundation_part_id(ctx)?;

        set_cannon_parts(ctx, 0, cannon_id, style_id, foundation_id)?;

        let kind = get_cannon_part_rec(ctx)?.kind;

        set_cannon_type(ctx, 0, kind)?;

        let wave = get_cannon_part_rec(ctx)?.makes_wave;

        set_cannon_makes_wave(ctx, 0, wave as i32)?;

        let base_damage = get_cannon_base_damage(ctx, 0)?;
        let scale = get_cannon_effect(get_cannon_part_rec(ctx)?, 0, level)?;

        set_cannon_damage(
            ctx,
            0,
            ops::div_100(scale.wrapping_mul(base_damage) as i64) as i32,
        )?;

        let recoil = get_cannon_part_rec(ctx)?.recoil;

        set_cannon_recoil(ctx, 0, recoil as i32)?;
        set_cannon_unit_id(ctx, 0, -3)?;

        let ready = get_cannon_part_rec(ctx)?.ready_vfx;

        set_cannon_ready_vfx(ctx, 0, ready as i32)?;
        set_cannon_hp_mode(ctx, 0, 0)?;

        let soulstrike = get_cannon_part_rec(ctx)?.soulstrike;

        set_base_soulstrike(ctx, 0, soulstrike as i32)?;

        let kind = get_cannon_part_rec(ctx)?.kind;

        get_cannon_part_rec(ctx)?;

        if kind == 2 {
            let value = get_cannon_effect(get_cannon_part_rec(ctx)?, 3, level)?;

            set_cannon_wall_hp_pct(ctx, 0, value)?;

            let value = get_cannon_effect(get_cannon_part_rec(ctx)?, 4, level)?;

            set_cannon_wall_lifetime(ctx, 0, value)?;

            let unit = get_cannon_part_rec(ctx)?.unit_id;

            set_cannon_unit_id(ctx, 0, unit)?;

            let value = get_cannon_effect(get_cannon_part_rec(ctx)?, 6, level)?;

            set_cannon_wall_offset(ctx, 0, value)?;
        } else {
            let kind = get_cannon_part_rec(ctx)?.kind;
            let unit = get_cannon_part_rec(ctx)?.unit_id;

            if kind == 4 {
                set_cannon_hp_mode(ctx, 0, unit)?;

                let value = get_cannon_effect(get_cannon_part_rec(ctx)?, 7, level)?;

                set_cannon_metal_permille(ctx, 0, value)?;

                let value = get_cannon_effect(get_cannon_part_rec(ctx)?, 8, level)?;

                set_cannon_nonmetal_permille(ctx, 0, value)?;
            } else if get_cannon_part_rec(ctx)?.kind == 5 {
                let unit = get_cannon_part_rec(ctx)?.unit_id;

                set_cannon_hp_mode(ctx, 0, unit)?;

                let value = get_cannon_effect(get_cannon_part_rec(ctx)?, 9, level)?;

                set_cannon_zombie_permille(ctx, 0, value)?;

                let value = get_cannon_effect(get_cannon_part_rec(ctx)?, 0xb, level)?;

                set_cannon_nonzombie_permille(ctx, 0, value)?;

                let value = get_cannon_effect(get_cannon_part_rec(ctx)?, 0xa, level)?;

                set_cannon_burrowed_permille(ctx, 0, value)?;
            }
        }

        if get_cannon_part_rec(ctx)?.kind == 3
            || get_cannon_part_rec(ctx)?.kind == 4
            || get_cannon_part_rec(ctx)?.kind == 6
        {
            let value = get_cannon_effect(get_cannon_part_rec(ctx)?, 5, level)?;

            set_cannon_strike_width(ctx, 0, value)?;
        }

        let slow = if get_cannon_effect(get_cannon_part_rec(ctx)?, 1, level)? > 0 {
            set_base_slow_chance(ctx, 0, 100)?;
            get_cannon_effect(get_cannon_part_rec(ctx)?, 1, level)?
        } else {
            set_base_slow_chance(ctx, 0, 0)?;
            0
        };

        set_base_slow_duration(ctx, 0, slow)?;

        let freeze = if get_cannon_effect(get_cannon_part_rec(ctx)?, 2, level)? > 0 {
            set_base_freeze_chance(ctx, 0, 100)?;
            get_cannon_effect(get_cannon_part_rec(ctx)?, 2, level)?
        } else {
            set_base_freeze_chance(ctx, 0, 0)?;
            0
        };

        set_base_freeze_duration(ctx, 0, freeze)?;

        let curse = if get_cannon_effect(get_cannon_part_rec(ctx)?, 0xc, level)? > 0 {
            set_base_curse_chance(ctx, 0, 100)?;
            get_cannon_effect(get_cannon_part_rec(ctx)?, 0xc, level)?
        } else {
            set_base_curse_chance(ctx, 0, 0)?;
            0
        };

        set_base_curse_duration(ctx, 0, curse)?;
        set_base_occupant(ctx, 1)?;
        set_base_state(ctx, 1, 0)?;
        set_base_entity_frame(ctx, 1, 0)?;
        set_base_pos_x(ctx, 1, 0xc80)?;
        set_base_pos_y(ctx, 1, 0x1130)?;

        let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;
        let multiplier = if mode >= 3 || ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0 {
            1
        } else {
            mode.wrapping_add(1)
        };
        let health = multiplier.wrapping_mul(ctx.i32_at(AppContext::STAGE_BASE_HP)?);

        set_base_max_hp(ctx, 1, health)?;

        let health = get_base_max_hp(ctx, 1)?;

        set_base_hp(ctx, 1, health)?;
        set_base_hitbox_pos(ctx, 1, 0)?;
        set_base_hitbox_width(ctx, 1, 0x708)?;

        let low = ctx.i32_at(AppContext::STAGE_SPAWN_MIN)?;
        let span = ctx
            .i32_at(AppContext::STAGE_SPAWN_MAX)?
            .wrapping_sub(low)
            .wrapping_add(1);
        let countdown = call_rng(ctx, span).wrapping_add(ctx.i32_at(AppContext::STAGE_SPAWN_MIN)?);

        ctx.set_i32_at(AppContext::SPAWN_COUNTDOWN, countdown)?;
        ctx.spawn_states.clear();

        for entry in 0..ctx.stage_enemies.len() {
            let mut state = [0i32; 3];

            spawn_state_init(&mut state);
            ctx.spawn_states.push(state);

            let start = stage_entry_start_frame(ctx.stage_enemies.get(entry).ok_or(
                Fault::index_out_of_range(entry as i64, 0),
            )?);

            ctx.spawn_states
                .get_mut(entry)
                .ok_or(Fault::index_out_of_range(entry as i64, 0))?[0] = start;
        }

        for slot in (0..0x380usize).step_by(0x10) {
            clear_debris(ctx, AppContext::CAT_DEBRIS + slot)?;
        }

        for slot in (0..0x380usize).step_by(0x10) {
            clear_debris(ctx, AppContext::ENEMY_DEBRIS + slot)?;
        }

        for shot in 0..0xfusize {
            clear_cannon_shot(ctx, AppContext::CANNON_SHOTS + shot * 0xc)?;
        }

        for slot in (0..0xc80usize).step_by(0x10) {
            clear_crit_vfx(ctx, AppContext::CRIT_VFX + slot)?;
        }

        for slot in 0..0x1eusize {
            clear_zkill_vfx(ctx, AppContext::ZKILL_VFX + slot * 0x10)?;
        }

        for slot in 0..0x1eusize {
            clear_barrier_vfx(ctx, AppContext::BARRIER_VFX + slot * 0x1c)?;
        }

        for slot in 0..0x1eusize {
            clear_shield_vfx(ctx, AppContext::SHIELD_VFX + slot * 0x1c)?;
        }

        ctx.savage_vfx.clear();
        ctx.toxic_vfx.clear();
        ctx.metal_killer_vfx.clear();
        ctx.drain_vfx.clear();
        clear_base_guard_notice(ctx)?;
        ctx.set_block_at::<8>(AppContext::BATTLE_CLOCK, [0; 8])?;
    }

    altar_recompute(ctx)?;

    let row = get_castle_enemy_row(ctx)?;
    let sealed =
        stage_not_sealed(ctx, row.wrapping_sub(2))? || ctx.i32_at(AppContext::BATTLE_RESUMED)? != 0;

    ctx.set_i32_at(
        AppContext::DEMON_BANNER_FRAME,
        (sealed as i32).wrapping_neg(),
    )?;

    if is_aku_final_map(ctx)? || is_ex_map_68(ctx)? || is_ex_option_target(ctx)? {
        let ex_map = ctx.i32_at(AppContext::EX_MAP)?;

        load_map_stage_csv(ctx, ex_map, 0, 1, 0, 1, 1)?;
    }

    ctx.set_block_at::<1>(AppContext::POWERUP_USED, [0])?;

    for powerup in [2, 3, 5] {
        if get_powerup(ctx, powerup)? {
            ctx.set_block_at::<1>(AppContext::POWERUP_USED, [1])?;
        }
    }

    let cleared =
        if ctx.i32_at(AppContext::CASTLE_ID)? == 0x2d {
            let row = ctx.i32_at(AppContext::STAGE_ROW)? as i64 as usize;
            let mut pair = [0u8; 8];

            pair[..4].copy_from_slice(&ctx.block_at::<4>(
                AppContext::STAGE_RECORD_CHAPTERS.wrapping_add(row.wrapping_mul(4)),
            )?);
            pair[4..].copy_from_slice(&ctx.block_at::<4>(AppContext::STAGE_RECORD_CHAPTERS_KEY)?);

            ops::xor_row_decode(&pair, 1, 0).ok_or(Fault::index_out_of_range(0, 1))? as i32
                > 0
        } else {
            false
        };

    ctx.set_block_at::<1>(AppContext::STAGE_CLEAR_FLAG, [cleared as u8])?;

    let row = ctx.i32_at(AppContext::STAGE_ROW)?;

    ctx.set_i32_at(AppContext::STAGE_MUSIC_ROW, row)?;

    let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;

    if mode as u32 <= 2 {
        if mode == 1 && row == 0x2f {
            ctx.set_i32_at(AppContext::STAGE_MUSIC_ROW, 0x31)?;
        } else if mode == 2 && row == 0x2f {
            ctx.set_i32_at(AppContext::STAGE_MUSIC_ROW, 0x32)?;
        }
    }

    if ctx.i32_at(AppContext::BATTLE_RESUMED)? == 0 {
        ctx.set_block_at::<0x28>(AppContext::FACTION_1_BUTTON_ROWS, [0xff; 0x28])?;

        for entry in 0..ctx.stage_enemies.len() {
            let row =
                stage_entry_row(ctx.stage_enemies.get(entry).ok_or(Fault::index_out_of_range(entry as i64, 0))?);

            for slot in 0..10usize {
                let current = ctx.i32_at(AppContext::FACTION_1_BUTTON_ROWS + slot * 4)?;

                if current == row {
                    break;
                }

                if current == -1 {
                    if slot < 9 || row != -1 {
                        ctx.set_i32_at(AppContext::FACTION_1_BUTTON_ROWS + slot * 4, row)?;
                    }

                    break;
                }
            }
        }
    }

    load_battle_assets(ctx)?;
    ctx.bg_models.clear();
    ctx.bg_anim_cache.clear();
    ctx.bg_model_names.clear();

    let mut index = 0;

    loop {
        let background = get_bg_model_id(ctx)?;
        let name = string_format_int2(ctx, b"bg%02d_%02d.mamodel", background, index)?;
        let mut model = ctx.bg_models.remove(&name).unwrap_or_default();
        let loaded = mamodel_load(ctx, &mut model, &name)?;

        ctx.bg_models.insert(name.clone(), model);

        if !loaded {
            break;
        }

        mamodel_set_single_sheet(ctx.bg_models.entry(name.clone()).or_default(), 1);
        *ctx.bg_model_names.entry(index).or_default() = name;
        index += 1;
    }

    ctx.bg_anim_names.clear();

    let mut index = 0;

    loop {
        let background = get_bg_model_id(ctx)?;
        let name = string_format_int2(ctx, b"bg%02d_%02d.maanim", background, index)?;
        let mut anim = ctx.bg_anim_cache.remove(&name).unwrap_or_default();
        let loaded = maanim_load(ctx, &mut anim, &name)?;

        ctx.bg_anim_cache.insert(name.clone(), anim);

        if !loaded {
            break;
        }

        *ctx.bg_anim_names.entry(index).or_default() = name;
        index += 1;
    }

    if ctx.i32_at(AppContext::BATTLE_RESUMED)? == 0 {
        background_particles_init(ctx)?;
    }

    bg_effect_spawn_all(ctx)?;

    let cannon = get_cannon_id(ctx, 0)?;
    let foundation = get_cannon_foundation_id(ctx, 0)?;
    let decor = get_cannon_decor_id(ctx, 0)?;

    load_base_models(ctx, 0, cannon, foundation, decor)?;
    load_base_models(ctx, 1, 0, 0, 0)?;
    scene_background_setup(ctx)?;
    option_window_init(ctx, ctx.img100_sheet.clone(), 1)?;

    for slot in ctx.label_texts.iter_mut() {
        *slot = None;
    }

    ctx.restriction_warning_texts = [None, None, None];
    ctx.set_block_at::<0x18>(AppContext::TOOLTIP_PAGE, [0; 0x18])?;
    ctx.restriction_warning_texts[0] = {
        let font = ctx.default_font.clone();
        let text = ctx
            .battle_texts
            .get(5)
            .cloned()
            .ok_or(Fault::index_out_of_range(5, 0x35))?;

        Some(get_text_texture(
            text_texture_cache(ctx)?,
            &text,
            &font,
            0x1e,
            1,
            0,
        ))
    };
    ctx.restriction_warning_texts[1] = {
        let key = std_string_from_cstr(b"stage_restriction_warning");
        let text = query_localizable(ctx, &key);
        let font = ctx.default_font.clone();

        Some(get_text_texture(
            text_texture_cache(ctx)?,
            &text,
            &font,
            0x1e,
            1,
            0,
        ))
    };
    ctx.restriction_warning_texts[2] = {
        let key = std_string_from_cstr(b"stage_SPChara_warning");
        let text = query_localizable(ctx, &key);
        let font = ctx.default_font.clone();

        Some(get_text_texture(
            text_texture_cache(ctx)?,
            &text,
            &font,
            0x1e,
            1,
            0,
        ))
    };

    for label in 0..3usize {
        ctx.label_texts[1 + label] =
            {
                let font = ctx.default_font.clone();
                let text = ctx.option_rows[1][label].clone();

                Some(get_text_texture(
                    text_texture_cache(ctx)?,
                    &text,
                    &font,
                    0x1e,
                    1,
                    0,
                ))
            };
    }

    for label in 0..3usize {
        ctx.label_texts[10 + label] =
            {
                let font = ctx.default_font.clone();
                let text = ctx.battle_option_texts.get(3 + label).cloned().ok_or(
                    Fault::index_out_of_range((3 + label) as i64, 9),
                )?;

                Some(get_text_texture(
                    text_texture_cache(ctx)?,
                    &text,
                    &font,
                    0x1e,
                    1,
                    0,
                ))
            };
    }

    let x = get_drawable_width(ctx)?
        .wrapping_sub(get_right_inset_logical(ctx)?)
        .wrapping_add(-0x92);

    ctx.set_i32_at(AppContext::CANNON_RECT, x)?;

    let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;

    ctx.set_i32_at(
        AppContext::CANNON_RECT + 4,
        get_top_inset_offset(ctx)
            .wrapping_add(shift)
            .wrapping_add(0x1fe),
    )?;
    ctx.set_i32_at(AppContext::CANNON_RECT + 8, 0xc2)?;
    ctx.set_i32_at(AppContext::CANNON_RECT + 0xc, 0x82)?;
    ctx.set_i32_at(
        AppContext::WORKER_RECT,
        get_left_inset_logical(ctx).wrapping_add(-0x30),
    )?;

    let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;

    ctx.set_i32_at(
        AppContext::WORKER_RECT + 4,
        get_top_inset_offset(ctx)
            .wrapping_add(shift)
            .wrapping_add(0x207),
    )?;
    ctx.set_i32_at(AppContext::WORKER_RECT + 8, 0xc2)?;
    ctx.set_i32_at(AppContext::WORKER_RECT + 0xc, 0x7d)?;
    ctx.set_i32_at(
        AppContext::COMBO_SKIP_RECT,
        get_drawable_width(ctx)?.wrapping_add(-0x5c),
    )?;
    ctx.set_i32_at(AppContext::COMBO_SKIP_RECT + 4, 0xa3)?;
    ctx.set_i32_at(AppContext::COMBO_SKIP_RECT + 8, 0x58)?;
    ctx.set_i32_at(AppContext::COMBO_SKIP_RECT + 0xc, 0x4e)?;
    ctx.set_i32_at(AppContext::PAUSE_RECT, get_left_inset_logical(ctx))?;
    ctx.set_i32_at(
        AppContext::PAUSE_RECT + 4,
        ctx.i32_at(AppContext::LETTERBOX_SHIFT)?.wrapping_neg(),
    )?;
    ctx.set_i32_at(AppContext::PAUSE_RECT + 8, 0x58)?;
    ctx.set_i32_at(AppContext::PAUSE_RECT + 0xc, 0x58)?;

    let mut column = 5i32;

    for powerup in (0..6i32).rev() {
        if !powerup_available(ctx, powerup)? {
            continue;
        }

        let rect = AppContext::ITEM_RECTS + powerup as usize * 0x10;
        let x = get_drawable_width(ctx)?
            .wrapping_add(column.wrapping_mul(0x58))
            .wrapping_sub(get_right_inset_logical(ctx)?)
            .wrapping_add(-0x210);

        ctx.set_i32_at(rect, x)?;
        ctx.set_i32_at(
            rect + 4,
            0x2bi32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?),
        )?;
        ctx.set_i32_at(rect + 8, 0x58)?;
        ctx.set_i32_at(rect + 0xc, 0x58)?;
        column -= 1;
    }

    if ctx.i32_at(AppContext::BATTLE_RESUMED)? == 0 {
        ctx.set_i32_at(AppContext::TOOLTIP_PAGE, 0x14)?;
        ctx.set_block_at::<0x10>(AppContext::SWIPE_VELOCITY, [0; 0x10])?;
        ctx.set_block_at::<0x2a>(AppContext::DECK_ROW_SHOWN, [0; 0x2a])?;
        ctx.set_block_at::<0x20>(AppContext::DECK_SWAP_BLOCK, [0; 0x20])?;
        set_auto_camera_mode(ctx, 0)?;
        ctx.set_i32_at(AppContext::SPEED, 1)?;
        ctx.set_block_at::<1>(AppContext::AUTO_CAMERA_ARRIVED, [0])?;
        ctx.set_i32_at(AppContext::LOSE_CHOICE, 0)?;
        ctx.set_i32_at(AppContext::CPU_PENDING_ACTION, 0)?;
        ctx.set_i32_at(AppContext::LOSE_TIP, 0)?;
        ctx.set_block_at::<0x10>(AppContext::TUTORIAL_TIMER, [0; 0x10])?;
        ctx.set_i32_at(AppContext::TUTORIAL_TIMER_TAIL, 0)?;
    }

    sound_manager(ctx)?.stop_audio(-1);

    if ctx.i32_at(AppContext::REVIVE_REQUESTED)? == 0 {
        let keep =
            ctx.i32_at(AppContext::BATTLE_RESUMED)? != 0 || ctx.u8_at(AppContext::EX_OFFERED)? != 0;

        bgm_player_switch(ctx, 0, keep as u8)?;
    }

    let resumed = ctx.i32_at(AppContext::BATTLE_RESUMED)?;

    if resumed > 0 {
        if ctx.i32_at(AppContext::REVIVE_REQUESTED)? == 0 {
            let state = ctx.i32_at(AppContext::BGM_SWITCH_STATE)?;

            if state == 0 {
                ctx.set_block_at::<0x10>(
                    AppContext::BGM_SWITCH_STATE,
                    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff],
                )?;
            } else if state == 1 {
                let frame = ctx.i32_at(AppContext::BGM_SWITCH_FRAME)?.wrapping_add(1);
                let frames = ctx.i32_at(AppContext::BGM_SWITCH_FRAMES)?;

                ctx.set_i32_at(AppContext::BGM_SWITCH_FRAME, frame)?;

                let left = frames.wrapping_sub(frame);

                if left != 0 {
                    let duck = ops::idiv(left.wrapping_mul(100), frames)
                        .ok_or(Fault::divide(frames as i64))?;

                    if duck > 0 {
                        let frames = ctx.i32_at(AppContext::BGM_SWITCH_FRAMES)?;
                        let left = frames.wrapping_sub(ctx.i32_at(AppContext::BGM_SWITCH_FRAME)?);
                        let duck = ops::idiv(left.wrapping_mul(100), frames)
                            .ok_or(Fault::divide(frames as i64))?;

                        set_bgm_duck(sound_manager(ctx)?, duck);
                    } else {
                        set_bgm_duck(sound_manager(ctx)?, 0);
                    }
                } else {
                    sound_manager(ctx)?.stop_audio(-1);

                    let next = ctx.i32_at(AppContext::BGM_SWITCH_NEXT)?;

                    if next != -1 {
                        play_sound(sound_manager(ctx)?, next, None);
                    }

                    ctx.set_block_at::<0x10>(
                        AppContext::BGM_SWITCH_STATE,
                        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff],
                    )?;
                }
            }
        }

        if ctx.u8_at(AppContext::CAT_GOD_MENU_IS_OPEN)? != 0 {
            bgm_player_switch(ctx, 1, 1)?;
        }
    }

    if ctx.i32_at(AppContext::BATTLE_RESUMED)? == 0 {
        ctx.set_i32_at(AppContext::DRAW_TEMP_0, 0)?;
        ctx.set_block_at::<8>(AppContext::CAT_GOD_SPIN, [0; 8])?;
        ctx.set_i32_at(AppContext::CAT_GOD_GLOW, 0)?;
        ctx.set_block_at::<0x30>(AppContext::SETUP_FRAMES, [0; 0x30])?;
        ctx.set_block_at::<1>(AppContext::INPUT_BLOCKED, [0])?;
        ctx.set_i32_at(AppContext::TOOLTIP_ITEM, 0)?;
        ctx.set_i32_at(AppContext::UI_STATE_TAIL, 0)?;
        ctx.set_block_at::<0x20>(AppContext::SCROLL_STATE, [0; 0x20])?;
        ctx.set_block_at::<0x2ba>(AppContext::CAMERA_KICK, [0; 0x2ba])?;
        ctx.set_block_at::<0x195>(AppContext::PENDING_STRIKE_SPEED, [0; 0x195])?;
        ctx.set_block_at::<0x509>(AppContext::SNIPER_FIRE_FRAME, [0; 0x509])?;
        ctx.set_block_at::<0x104>(AppContext::HUD_RECTS, [0; 0x104])?;
    }

    let rects: [(usize, i32, [i32; 3]); 12] = [
        (0x0, 0x118, [0x50, 0x58, 0x58]),
        (0x20, 0x217, [0x114, 0x6a, 0x58]),
        (0x30, 0x1f2, [0x173, 0x58, 0x58]),
        (0x40, 0x25a, [0x173, 0x58, 0x58]),
        (0x50, 0x28c, [0x30, 0x5f, 0x5f]),
        (0x60, 0x120, [0x1c8, 0x17d, 0x58]),
        (0x70, 0x120, [0xbd, 0xa8, 0x58]),
        (0x80, 0x135, [0x11c, 0x83, 0x83]),
        (0x90, 0x120, [0x16b, 0xa8, 0x58]),
        (0xa0, 0xfb, [0x176, 0xa8, 0x58]),
        (0xb0, 0x21d, [0x176, 0xa8, 0x58]),
        (0xc0, 0x1f4, [0xbf, 0xac, 0x58]),
    ];

    for (index, (offset, shift, rest)) in rects.iter().enumerate() {
        let rect = AppContext::HUD_RECTS + offset;

        ctx.set_i32_at(
            rect,
            ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(*shift),
        )?;
        ctx.set_i32_at(rect + 4, rest[0])?;
        ctx.set_i32_at(rect + 8, rest[1])?;
        ctx.set_i32_at(rect + 0xc, rest[2])?;

        if index == 0 {
            ctx.set_block_at::<0x10>(rect + 0x10, [0; 0x10])?;
        }
    }

    ctx.set_i32_at(
        AppContext::CAT_GOD_BUTTON_RECT,
        ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0xf6),
    )?;
    ctx.set_i32_at(
        AppContext::CAT_GOD_BUTTON_RECT + 4,
        ctx.i32_at(AppContext::LETTERBOX_SHIFT)?.wrapping_neg(),
    )?;
    ctx.set_i32_at(AppContext::CAT_GOD_BUTTON_RECT + 8, 0x83)?;
    ctx.set_i32_at(AppContext::CAT_GOD_BUTTON_RECT + 0xc, 0x6b)?;

    let menu: [(usize, i32, [i32; 3]); 3] = [
        (0, 0xf6, [0x159, 0x60, 0x60]),
        (0x10, 0x1aa, [0x14f, 0x60, 0x60]),
        (0x20, 0x25e, [0x159, 0x60, 0x60]),
    ];

    for (offset, shift, rest) in menu {
        let rect = AppContext::CAT_GOD_MIRACLE_RECTS + offset;

        ctx.set_i32_at(
            rect,
            ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(shift),
        )?;
        ctx.set_i32_at(rect + 4, rest[0])?;
        ctx.set_i32_at(rect + 8, rest[1])?;
        ctx.set_i32_at(rect + 0xc, rest[2])?;
    }

    ctx.set_i32_at(
        AppContext::CAT_GOD_MIRACLE_RECTS + 0x30,
        ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x312),
    )?;
    ctx.set_i32_at(AppContext::CAT_GOD_MIRACLE_RECTS + 0x34, 0x14f)?;
    ctx.set_i32_at(AppContext::CAT_GOD_MIRACLE_RECTS + 0x38, 0x60)?;
    ctx.set_i32_at(AppContext::CAT_GOD_MIRACLE_RECTS + 0x3c, 0x60)?;
    ctx.set_i32_at(AppContext::CAT_GOD_CLOSE_RECT, 4)?;
    ctx.set_i32_at(AppContext::CAT_GOD_CLOSE_RECT + 4, 0x21d)?;
    ctx.set_i32_at(AppContext::CAT_GOD_CLOSE_RECT + 8, 0x5f)?;
    ctx.set_i32_at(AppContext::CAT_GOD_CLOSE_RECT + 0xc, 0x5f)?;
    ctx.set_i32_at(
        AppContext::CAT_GOD_CONFIRM_RECT,
        ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x1a6),
    )?;
    ctx.set_i32_at(AppContext::CAT_GOD_CONFIRM_RECT + 4, 0x139)?;
    ctx.set_i32_at(AppContext::CAT_GOD_CONFIRM_RECT + 8, 0x17d)?;
    ctx.set_i32_at(AppContext::CAT_GOD_CONFIRM_RECT + 0xc, 0x58)?;
    ctx.set_i32_at(
        AppContext::CAT_GOD_BACK_RECT,
        ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(0x323),
    )?;
    ctx.set_i32_at(AppContext::CAT_GOD_BACK_RECT + 4, 0xad)?;
    ctx.set_i32_at(AppContext::CAT_GOD_BACK_RECT + 8, 0x5f)?;
    ctx.set_i32_at(AppContext::CAT_GOD_BACK_RECT + 0xc, 0x5f)?;
    ctx.set_i32_at(
        AppContext::CAT_GOD_BACK_RECT + 0x10,
        get_drawable_width(ctx)?.wrapping_add(-0x17c),
    )?;
    ctx.set_i32_at(AppContext::CAT_GOD_BACK_RECT + 0x14, 0x223)?;
    ctx.set_i32_at(AppContext::CAT_GOD_BACK_RECT + 0x18, 0x58)?;
    ctx.set_i32_at(AppContext::CAT_GOD_BACK_RECT + 0x1c, 0x58)?;
    ctx.set_i32_at(
        AppContext::OPTION_RECTS,
        get_drawable_width(ctx)?.wrapping_add(-0x103),
    )?;

    let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;

    let inset = get_bottom_inset_logical(ctx)?;

    ctx.set_i32_at(
        AppContext::OPTION_RECTS + 4,
        shift.wrapping_sub(inset).wrapping_add(0x22e),
    )?;
    ctx.set_i32_at(AppContext::OPTION_RECTS + 8, 0x58)?;
    ctx.set_i32_at(AppContext::OPTION_RECTS + 0xc, 0x58)?;
    ctx.set_i32_at(
        AppContext::OPTION_RECTS + 0x10,
        get_drawable_width(ctx)?.wrapping_add(-0xa1),
    )?;

    let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;

    let inset = get_bottom_inset_logical(ctx)?;

    ctx.set_i32_at(
        AppContext::OPTION_RECTS + 0x14,
        shift.wrapping_sub(inset).wrapping_add(0x22e),
    )?;
    ctx.set_i32_at(AppContext::OPTION_RECTS + 0x18, 0x58)?;
    ctx.set_i32_at(AppContext::OPTION_RECTS + 0x1c, 0x58)?;
    ctx.set_i32_at(AppContext::OUTRO_RECTS, 0xc5)?;
    ctx.set_i32_at(AppContext::OUTRO_RECTS + 4, 0x228)?;
    ctx.set_i32_at(AppContext::OUTRO_RECTS + 8, 0xd6)?;
    ctx.set_i32_at(AppContext::OUTRO_RECTS + 0xc, 0x58)?;
    ctx.set_i32_at(
        AppContext::LOSE_SHOP_RECT,
        get_drawable_width(ctx)?.wrapping_add(-0x118),
    )?;
    ctx.set_i32_at(AppContext::LOSE_SHOP_RECT + 4, 0x228)?;
    ctx.set_i32_at(AppContext::LOSE_SHOP_RECT + 8, 0x58)?;
    ctx.set_i32_at(AppContext::LOSE_SHOP_RECT + 0xc, 0x58)?;

    let resumed = ctx.i32_at(AppContext::BATTLE_RESUMED)?;

    if resumed > 0 {
        if get_battle_status(ctx)? == 2 {
            for label in 0..4usize {
                ctx.label_texts[label] = {
                    let text = ctx
                        .warning2_rows
                        .get(2)
                        .map(|row| row[label].clone())
                        .ok_or(Fault::index_out_of_range(2, 0))?;
                    let font = ctx.default_font.clone();

                    Some(get_text_texture(
                        text_texture_cache(ctx)?,
                        &text,
                        &font,
                        0x1e,
                        1,
                        0,
                    ))
                };
            }
        } else if ctx.u8_at(AppContext::CAT_GOD_MENU_IS_OPEN)? != 0 {
            for slot in ctx.label_texts.iter_mut() {
                *slot = None;
            }

            ctx.set_i32_at(AppContext::CAT_GOD_CHATTER_TIMER, 0x12c)?;
            ctx.label_texts[0] = {
                let font = ctx.default_font.clone();
                let text = ctx
                    .god_intro_texts
                    .first()
                    .map(|row| row[0].clone())
                    .ok_or(Fault::index_out_of_range(0, 0))?;

                Some(get_text_texture(
                    text_texture_cache(ctx)?,
                    &text,
                    &font,
                    0x1e,
                    1,
                    0,
                ))
            };
            ctx.label_texts[1] = {
                let font = ctx.default_font.clone();
                let text = ctx
                    .god_intro_texts
                    .first()
                    .map(|row| row[1].clone())
                    .ok_or(Fault::index_out_of_range(0, 0))?;

                Some(get_text_texture(
                    text_texture_cache(ctx)?,
                    &text,
                    &font,
                    0x1e,
                    1,
                    0,
                ))
            };
            ctx.label_texts[2] = {
                let font = ctx.default_font.clone();
                let text = ctx.god_name_text.clone();

                Some(get_text_texture(
                    text_texture_cache(ctx)?,
                    &text,
                    &font,
                    0x1e,
                    1,
                    0,
                ))
            };

            if ctx.i32_at(AppContext::CAT_GOD_STATE)? == 4 {
                ctx.label_texts[0] = {
                    let font = ctx.default_font.clone();
                    let text = ctx.god_bought_texts[0].clone();

                    Some(get_text_texture(
                        text_texture_cache(ctx)?,
                        &text,
                        &font,
                        0x1e,
                        1,
                        0,
                    ))
                };
                ctx.label_texts[1] = {
                    let font = ctx.default_font.clone();
                    let text = ctx.god_bought_texts[1].clone();

                    Some(get_text_texture(
                        text_texture_cache(ctx)?,
                        &text,
                        &font,
                        0x1e,
                        1,
                        0,
                    ))
                };
            }
        }
    }

    if ctx.i32_at(AppContext::BATTLE_RESUMED)? == 0 && has_castle_enemy(ctx)? {
        for entry in 0..ctx.stage_enemies.len() {
            let row =
                stage_entry_row(ctx.stage_enemies.get(entry).ok_or(Fault::index_out_of_range(entry as i64, 0))?);

            if row != get_castle_enemy_row(ctx)? {
                continue;
            }

            let enemy = ctx.stage_enemies.get(entry).ok_or(Fault::index_out_of_range(entry as i64, 0))?;
            let row = stage_entry_row(enemy);
            let z_min = stage_entry_z_min(enemy);
            let z_max = stage_entry_z_max(enemy);

            spawn_entity(ctx, 1, row, 0, z_min, z_max, 0, entry as i32)?;

            let state = ctx
                .spawn_states
                .get_mut(entry)
                .ok_or(Fault::index_out_of_range(entry as i64, 0))?;

            state[0] = 1;
            state[1] = state[1].wrapping_add(1);
            break;
        }
    }

    if ctx.i32_at(AppContext::BATTLE_RESUMED)? == 0 {
        let map_id = get_global_map_id(ctx, 0)?;

        if ctx.map_records.contains_key(&map_id) {
            let map_id = get_global_map_id(ctx, 0)?;
            let voice = ctx.map_records.entry(map_id).or_default().start_voice;

            if voice != -1 {
                let map_id = get_global_map_id(ctx, 0)?;
                let voice = ctx.map_records.entry(map_id).or_default().start_voice;

                play_sound(sound_manager(ctx)?, voice, None);
            }
        }

        if ctx.i32_at(AppContext::BATTLE_RESUMED)? == 0 && get_map_type(ctx, 0)? != -10 {
            let map = get_global_map_id(ctx, 0)?;
            let stage = get_stage_index(ctx)?;
            let star = get_star_level(ctx)?;
            let leadership = ctx.i32_at(AppContext::LEADERSHIP_TOTAL)?;

            analytics_params_send(
                ctx,
                0x1317f09,
                leadership,
                &[
                    (b"sec1_type", FormatArg::Text(b"MapID")),
                    (b"sec1_id", FormatArg::Int(map)),
                    (b"sec2_type", FormatArg::Text(b"StageIdx")),
                    (b"sec2_id", FormatArg::Int(stage)),
                    (b"ex_type", FormatArg::Text(b"StageLv")),
                    (b"ex_id", FormatArg::Int(star)),
                ],
            )?;
        }
    }

    if get_stage_record(ctx, -2, 0, 3, 0, 0)? > 0
        && ctx.i32_at(AppContext::STAGE_NO_CONTINUES)? == 0
    {
        let eligible = ctx.i32_at(AppContext::CHAPTER_MODE)? != 0x63 || is_ex_option_target(ctx)?;

        if eligible && ctx.u8_at(AppContext::SCORE_MODE_FLAG)? == 0 && get_map_type(ctx, 0)? != -6 {
            ctx.set_i32_at(AppContext::BATTLE_RESUMED, 1)?;
            ctx.set_i32_at(AppContext::BATTLE_CONTINUED, 1)?;
            save_battle_snapshot(ctx)?;
        }
    }

    let mut units: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();

    for slot in 0..10usize {
        let key = string_format_int(ctx, b"Unit%d", slot as i32)?;
        let row = ctx.bytes_from(AppContext::BATTLE_DECK)?;
        let unit = ops::xor_row_decode(row, 10, slot).ok_or(Fault::index_out_of_range(slot as i64, 10))? as i32;
        let value = if unit > 0 {
            let form = ctx.i32_at(AppContext::BUTTON_UNIT_FORMS + slot * 4)?;

            string_format_int2(ctx, b"%03d,%d", unit, form)?
        } else {
            b"None".to_vec()
        };

        if !units.iter().any(|(name, _)| *name == key) {
            units.push((key, value));
        }
    }

    units.sort();

    let pairs: Vec<(&[u8], &[u8])> = units
        .iter()
        .map(|(name, value)| (name.as_slice(), value.as_slice()))
        .collect();

    breadcrumb_with(ctx, 0x46, &pairs)
}
