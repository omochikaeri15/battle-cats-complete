use crate::Fault;

use super::{
    AppContext, AssetStream, calculate_treasure_percentages, get_map_count, has_map_data_csv,
    load_altar_limit_csv, load_autoset_lineup_files, load_base_shake_csv, load_cat_cannon_csv,
    load_cat_combat_csv, load_cat_data_files, load_cat_drops_csv, load_cat_group_csv,
    load_catseye_behavior_json, load_change_conditions_csv, load_continue_stages_csv,
    load_daily_login_grade_json, load_dojo_chest_tsv, load_dojo_score_json, load_enemy_combat_csv,
    load_event_item_json, load_ex_option_csv, load_gacha_setting_csv, load_gatya_ability_csv,
    load_gold_cpu_csv, load_hidden_data_csv, load_item_pack_tsv, load_leadership_return_csv,
    load_lineup_csvs, load_map_layout, load_map_option_csv, load_map_option_json,
    load_map_stage_next, load_medal_data_file, load_officers_club_csv, load_orb_effect_csv,
    load_parameter_table_tsv, load_point_event_reward_json, load_point_files,
    load_point_release_json, load_realms_rng_csv, load_reccomended_levelup_csv,
    load_recommended_powerup_csv, load_slot_unlock_csv, load_sound_settings_tsv,
    load_special_rules_json, load_stage_filter_csv, load_talent_orb_files, load_talent_type_csv,
    load_tower_checkpoint_csv, load_treasure_data_csv, load_vibration_csv, map_type_as_index,
    open_asset_stream, read_csv_cell, read_csv_row, sound_manager, string_format_int,
};

const MAP_TYPES: [i32; 15] = [
    -4, -6, -9, -10, -11, -16, -17, -18, -8, -19, -20, -21, -22, -23, -26,
];

pub fn initialize_game_data(ctx: &mut AppContext) -> Result<bool, Fault> {
    if let Some(bytes) = open_asset_stream(ctx, b"StampData.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        for slot in 0..0x1fusize {
            read_csv_row(stm);

            let first = read_csv_cell(stm, 0) as i32;
            let second = read_csv_cell(stm, 1) as i32;

            ctx.set_i32_at(AppContext::STAMP_DATA + slot * 8, first)?;
            ctx.set_i32_at(AppContext::STAMP_DATA + slot * 8 + 4, second)?;
        }
    }

    load_gatya_ability_csv(ctx)?;

    let mut map_index = 0i32;

    while has_map_data_csv(ctx, map_index)? {
        if !load_map_layout(ctx, map_index)? {
            ctx.set_i32_at(AppContext::SCENE_4_PAGE, 8)?;

            return Ok(false);
        }

        map_index = map_index.wrapping_add(1);
    }

    let unitbuy = open_asset_stream(ctx, b"unitbuy.csv", 0, 0)?.unwrap_or_default();
    let unitlevel = open_asset_stream(ctx, b"unitlevel.csv", 0, 0)?.unwrap_or_default();
    let unitexp = open_asset_stream(ctx, b"unitexp.csv", 0, 0)?.unwrap_or_default();

    load_cat_data_files(
        ctx,
        &mut AssetStream::new(&unitbuy, b'\n'),
        &mut AssetStream::new(&unitlevel, b'\n'),
        &mut AssetStream::new(&unitexp, b'\n'),
    )?;

    for unit in 0..0x36ci32 {
        let name = string_format_int(ctx, b"unit%03d.csv", unit.wrapping_add(1))?;

        if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
            load_cat_combat_csv(ctx, &mut AssetStream::new(&bytes, b'\n'), unit)?;
        }
    }

    load_talent_type_csv(ctx)?;
    let t_unit = open_asset_stream(ctx, b"t_unit.csv", 0, 0)?.unwrap_or_default();

    load_enemy_combat_csv(ctx, &mut AssetStream::new(&t_unit, b'\n'))?;
    ctx.set_i32_at(AppContext::ALL_MAPS_OPEN, 0)?;

    for chapter in 0..5i32 {
        ctx.set_i32_at(AppContext::SAVED_MAP_TYPE, chapter)?;

        for map in 0..0x1f4i32 {
            load_map_stage_next(ctx, map, 0, 0, 0)?;
        }
    }

    for kind in MAP_TYPES {
        ctx.set_i32_at(AppContext::SAVED_MAP_TYPE, map_type_as_index(kind))?;

        if kind == -6 {
            load_map_stage_next(ctx, 0, 0, 0, 0)?;

            continue;
        }

        if kind == -8 {
            load_map_stage_next(ctx, 0x1a, 0, 0, 0)?;
            load_map_stage_next(ctx, 0x45, 0, 0, 0)?;

            continue;
        }

        let mut map = 0i32;

        while map < get_map_count(ctx, kind)? {
            load_map_stage_next(ctx, map, 0, 0, 0)?;

            map = map.wrapping_add(1);
        }
    }

    ctx.set_i32_at(AppContext::SAVED_MAP_TYPE, 0)?;
    ctx.set_i32_at(AppContext::ALL_MAPS_OPEN, 1)?;

    if !load_treasure_data_csv(ctx)? {
        return Ok(false);
    }

    calculate_treasure_percentages(ctx)?;
    load_cat_drops_csv(ctx)?;
    load_map_option_csv(ctx)?;
    load_item_pack_tsv(ctx)?;
    load_cat_cannon_csv(ctx)?;
    load_cat_group_csv(ctx)?;
    sound_manager(ctx)?;
    load_sound_settings_tsv(ctx, b"Sound_setting.tsv")?;
    load_officers_club_csv(ctx)?;
    load_medal_data_file(ctx)?;
    load_hidden_data_csv(ctx)?;
    load_dojo_chest_tsv(ctx)?;
    load_talent_orb_files(ctx)?;
    load_orb_effect_csv(ctx, 0)?;
    load_orb_effect_csv(ctx, 1)?;
    load_slot_unlock_csv(ctx)?;
    load_stage_filter_csv(ctx)?;
    load_map_option_json(ctx)?;
    load_parameter_table_tsv(ctx)?;
    load_altar_limit_csv(ctx)?;
    load_realms_rng_csv(ctx)?;
    load_continue_stages_csv(ctx)?;
    load_tower_checkpoint_csv(ctx)?;
    load_base_shake_csv(ctx)?;
    load_change_conditions_csv(ctx)?;
    load_catseye_behavior_json(ctx)?;
    load_gold_cpu_csv(ctx)?;
    load_recommended_powerup_csv(ctx)?;
    load_special_rules_json(ctx)?;

    let mut points = ctx.event_items.take().unwrap_or_default();

    load_point_files(ctx, &mut points)?;
    ctx.event_items = Some(points);

    load_event_item_json(ctx)?;
    load_gacha_setting_csv(ctx)?;
    load_ex_option_csv(ctx)?;
    load_lineup_csvs(ctx)?;
    load_dojo_score_json(ctx)?;
    load_leadership_return_csv(ctx)?;
    load_autoset_lineup_files(ctx)?;
    load_reccomended_levelup_csv(ctx)?;
    load_vibration_csv(ctx)?;
    load_daily_login_grade_json(ctx)?;
    load_point_event_reward_json(ctx)?;
    load_point_release_json(ctx)?;

    Ok(true)
}
