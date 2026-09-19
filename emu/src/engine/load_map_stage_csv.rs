use crate::Fault;

use super::{
    aku_realm_final_redirect, ex_redirect_check_a, ex_redirect_check_b, ex_redirect_check_c, ex_replacement_pending, get_ex_option_target,
    get_global_map_id, get_map_index, get_map_type, get_stage_count, get_stage_set_size, invasion_available, invasion_z_available,
    load_enemy_castle_csv, load_ex_map_stage_csv, map_index_of_map_id, map_type_as_index, map_type_base_id, open_asset_stream,
    pack_entry_text, read_csv_cell, read_csv_row, set_scene, string_format_int, string_format_int2, string_format_int2_text, validate_map_type,
    AppContext, AssetStream,
};

const SITE: &str = "load_map_stage_csv";

const MAP_FILES: [(i32, &[u8]); 16] = [
    (-4, b"MapStageDataV_%03d.csv"),
    (-6, b"MapStageDataM_%03d.csv"),
    (-9, b"MapStageDataNA_%03d.csv"),
    (-10, b"MapStageDataB_%03d.csv"),
    (-11, b"MapStageDataD_%03d.csv"),
    (-16, b"MapStageDataA_%03d.csv"),
    (-17, b"MapStageDataH_%03d.csv"),
    (-18, b"MapStageDataCA_%03d.csv"),
    (-8, b"MapStageDataRE_%03d.csv"),
    (-19, b"MapStageDataDM_%03d.csv"),
    (-20, b"MapStageDataQ_%03d.csv"),
    (-21, b"MapStageDataL_%03d.csv"),
    (-22, b"MapStageDataND_%03d.csv"),
    (-23, b"MapStageDataSR_%03d.csv"),
    (-24, b"MapStageDataG_%03d.csv"),
    (-26, b"MapStageDataPR_%03d.csv"),
];

const LEGEND_FILES: [&[u8]; 5] = [
    b"MapStageDataN_%03d.csv",
    b"MapStageDataS_%03d.csv",
    b"MapStageDataC_%03d.csv",
    b"MapStageDataT_%03d.csv",
    b"MapStageDataR_%03d.csv",
];

pub fn load_map_stage_csv(ctx: &mut AppContext, map: i32, stage: i32, check_pack: u8, saga: u8, ex: u8, redirect: u8) -> Result<bool, Fault> {
    if check_pack != 0 {
        let key = ctx.data_pack_key.clone();
        let entry = pack_entry_text(ctx, &key)?;

        if entry != ctx.data_pack_expected {
            ctx.set_i32_at(AppContext::SCENE_4_PAGE, 0x15)?;
            set_scene(ctx, 4)?;

            return Ok(false);
        }
    }

    if ex != 0 {
        return load_ex_map_stage_csv(ctx, map);
    }

    if saga != 0 {
        ctx.set_i32_at(AppContext::EVENT_REWARD_ID, -1)?;
        ctx.set_i32_at(AppContext::RANKING_ID, -1)?;

        let (bytes, is_saga) = if map == 2 {
            load_enemy_castle_csv(ctx, 3)?;

            let name = string_format_int2(ctx, b"stageNormal%d_%d.csv", map, stage)?;

            (open_asset_stream(ctx, &name, 0, 0)?.unwrap_or_default(), false)
        } else {
            ctx.set_block_at::<0x300>(AppContext::MAP_COORDS, [0; 0x300])?;

            let name = string_format_int(ctx, b"mapCordData%d.csv", map)?;

            if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
                let mut stm = AssetStream::new(&bytes, b'\n');

                for row in 0..0x30usize {
                    read_csv_row(&mut stm);

                    for col in 0..4usize {
                        ctx.set_i32_at(AppContext::MAP_COORDS + row * 0x10 + col * 4, read_csv_cell(&stm, col as i32) as i32)?;
                    }
                }
            }

            match map {
                1 => {
                    load_enemy_castle_csv(ctx, 1)?;

                    let name = string_format_int2(ctx, b"stageNormal%d_%d.csv", map, stage)?;

                    (open_asset_stream(ctx, &name, 0, 0)?.unwrap_or_default(), false)
                }
                0 => {
                    load_enemy_castle_csv(ctx, 0)?;

                    let name = string_format_int(ctx, b"stageNormal%d.csv", 0)?;

                    (open_asset_stream(ctx, &name, 0, 0)?.unwrap_or_default(), true)
                }
                _ => (Vec::new(), false),
            }
        };

        let mut stm = AssetStream::new(&bytes, b'\n');

        for row in 0..0x64usize {
            let row_at = AppContext::MAP_STAGE_ROWS + row * 0xbc;

            for col in 0..0x2eusize {
                let key = ctx.i32_at(row_at + 0xb8)?;

                ctx.set_i32_at(row_at + col * 4, !key)?;
            }
        }

        read_csv_row(&mut stm);
        read_csv_row(&mut stm);

        for row in 0..0x64usize {
            read_csv_row(&mut stm);

            let row_at = AppContext::MAP_STAGE_ROWS + row * 0xbc;

            for col in 0..0x2eusize {
                let value = read_csv_cell(&stm, col as i32) as i32;
                let key = ctx.i32_at(row_at + 0xb8)?;

                ctx.set_i32_at(row_at + col * 4, value ^ key)?;

                if ctx.i32_at(row_at + col * 4)? ^ ctx.i32_at(row_at + 0xb8)? == -1 {
                    break;
                }
            }

            if ctx.i32_at(row_at)? ^ ctx.i32_at(row_at + 0xb8)? == -1 {
                break;
            }
        }

        ctx.set_block_at::<1>(AppContext::OUTBREAKS_ENABLED, [1])?;

        let chapter = if is_saga { stage } else { map.wrapping_mul(3).wrapping_add(stage).wrapping_add(1) };

        if !ctx.outbreak_active.is_empty() {
            let name = string_format_int2(ctx, b"stageNormal%d_%d_Z.csv", map, stage)?;

            if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
                let mut stm = AssetStream::new(&bytes, b'\n');

                read_csv_row(&mut stm);
                read_csv_row(&mut stm);

                let mut row = 0usize;

                loop {
                    read_csv_row(&mut stm);

                    let active = *ctx.outbreak_active.entry(chapter).or_default().entry(row as i32).or_default();

                    if active {
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
                    } else if row > 0x62 {
                        break;
                    }

                    row += 1;
                }
            }
        }

        let chapter = if is_saga { stage } else { map.wrapping_mul(3).wrapping_add(stage).wrapping_add(1) };

        if (invasion_available(ctx, chapter)? || invasion_z_available(ctx, chapter)?) && ctx.u8_at(AppContext::INVASION_STAGE)? != 0xff {
            let suffix: &[u8] = if invasion_available(ctx, chapter)? { b"" } else { b"_Z" };
            let name = string_format_int2_text(ctx, b"stageNormal%d_%d_Invasion%s.csv", map, stage, suffix)?;

            if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
                let mut stm = AssetStream::new(&bytes, b'\n');

                read_csv_row(&mut stm);
                read_csv_row(&mut stm);
                read_csv_row(&mut stm);

                if ctx.u8_at(AppContext::INVASION_STAGE)? <= 0x63 {
                    for col in 0..0x2eusize {
                        let value = read_csv_cell(&stm, col as i32) as i32;
                        let row = ctx.u8_at(AppContext::INVASION_STAGE)? as i8 as isize as usize;
                        let row_at = AppContext::MAP_STAGE_ROWS.wrapping_add(row.wrapping_mul(0xbc));
                        let key = ctx.i32_at(row_at + 0xb8)?;

                        ctx.set_i32_at(row_at + col * 4, value ^ key)?;

                        let row = ctx.u8_at(AppContext::INVASION_STAGE)? as i8 as isize as usize;
                        let row_at = AppContext::MAP_STAGE_ROWS.wrapping_add(row.wrapping_mul(0xbc));

                        if ctx.i32_at(row_at + col * 4)? ^ ctx.i32_at(row_at + 0xb8)? == -1 {
                            break;
                        }
                    }
                }
            }
        }

        return Ok(true);
    }

    let map_type = ctx.i32_at(AppContext::SAVED_MAP_TYPE)?;
    let name = if (0..=4).contains(&map_type) {
        if ctx.u8_at(AppContext::ALL_MAPS_OPEN)? == 0 && ctx.i32_at(AppContext::STORY_MAP_COUNTS + map_type as usize * 4)? <= map {
            return Ok(false);
        }

        let name = string_format_int(ctx, LEGEND_FILES[map_type as usize], map)?;

        if map_type == 0 && ex_redirect_check_a(ctx)? && redirect != 0 {
            b"MapStageDataRE_026.csv".to_vec()
        } else {
            name
        }
    } else {
        let mut picked = None;

        for (kind, pattern) in MAP_FILES {
            if map_type_as_index(kind) == ctx.i32_at(AppContext::SAVED_MAP_TYPE)? {
                picked = Some((kind, pattern));
                break;
            }
        }

        let Some((kind, pattern)) = picked else {
            return Ok(false);
        };

        let name = string_format_int(ctx, pattern, map)?;

        if kind == -9 && ex_redirect_check_b(ctx)? && redirect != 0 {
            b"MapStageDataRE_069.csv".to_vec()
        } else {
            name
        }
    };

    let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? else {
        return Ok(true);
    };
    let mut stm = AssetStream::new(&bytes, b'\n');

    ctx.set_i32_at(AppContext::EVENT_REWARD_ID, -1)?;
    ctx.set_i32_at(AppContext::RANKING_ID, -1)?;
    read_csv_row(&mut stm);
    ctx.set_i32_at(AppContext::MAP_DATA_ID, read_csv_cell(&stm, 0) as i32)?;

    let options = [read_csv_cell(&stm, 3) as i32, read_csv_cell(&stm, 4) as i32, read_csv_cell(&stm, 5) as i32, read_csv_cell(&stm, 6) as i32];
    let reward = read_csv_cell(&stm, 1) as i32;
    let reward = if reward < 0x190 { reward } else { -1 };

    ctx.set_i32_at(AppContext::EVENT_REWARD_ID, reward)?;

    if reward != -1 {
        let base = map_type_base_id(validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?), map);

        *ctx.event_reward_maps.entry(base).or_default() = reward;
    }

    let ranking = read_csv_cell(&stm, 2) as i32;
    let ranking = if ranking < 0xc8 { ranking } else { -1 };

    ctx.set_i32_at(AppContext::RANKING_ID, ranking)?;

    if ranking != -1 {
        let base = map_type_base_id(validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?), map);

        *ctx.ranking_maps.entry(base).or_default() = ranking;
    }

    read_csv_row(&mut stm);
    ctx.set_i32_at(AppContext::MAP_STAGE_SET, read_csv_cell(&stm, 0) as i32)?;

    if ctx.u8_at(AppContext::ALL_MAPS_OPEN)? == 0 {
        let slot = map as i64 as usize;
        let data_id = ctx.i32_at(AppContext::MAP_DATA_ID)?;
        let kind = validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?);

        *ctx.map_data_ids.entry(kind).or_insert_with(|| vec![0; 0x1f4]).get_mut(slot).ok_or(Fault::IndexOutOfRange { site: SITE, index: map as i64, limit: 0x1f4 })? = data_id;

        let set = ctx.i32_at(AppContext::MAP_STAGE_SET)?;
        let kind = validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?);

        *ctx.map_stage_sets.entry(kind).or_insert_with(|| vec![0; 0x1f4]).get_mut(slot).ok_or(Fault::IndexOutOfRange { site: SITE, index: map as i64, limit: 0x1f4 })? = set;

        for (column, value) in options.into_iter().enumerate() {
            let kind = validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?);
            let row = ctx
                .map_stage_options
                .entry(kind)
                .or_insert_with(|| vec![[0; 4]; 0x1f4])
                .get_mut(slot)
                .ok_or(Fault::IndexOutOfRange { site: SITE, index: map as i64, limit: 0x1f4 })?;

            row[column] = value;
        }
    }

    for row in 0..0x64usize {
        let row_at = AppContext::MAP_STAGE_ROWS + row * 0xbc;

        for col in 0..0x2eusize {
            let key = ctx.i32_at(row_at + 0xb8)?;

            ctx.set_i32_at(row_at + col * 4, !key)?;
        }
    }

    let mut row = 0usize;

    loop {
        let data_id = ctx.i32_at(AppContext::MAP_DATA_ID)?;
        let set = ctx.i32_at(AppContext::MAP_STAGE_SET)?;
        let size = get_stage_set_size(ctx.map_data.entry(data_id).or_default(), set)? as i32 as i64;

        if row as i64 >= size {
            break;
        }

        read_csv_row(&mut stm);

        let row_at = AppContext::MAP_STAGE_ROWS.wrapping_add(row.wrapping_mul(0xbc));

        for col in 0..0x2eusize {
            let value = read_csv_cell(&stm, col as i32) as i32;
            let key = ctx.i32_at(row_at + 0xb8)?;

            ctx.set_i32_at(row_at + col * 4, value ^ key)?;

            if ctx.i32_at(row_at + col * 4)? ^ ctx.i32_at(row_at + 0xb8)? == -1 {
                break;
            }
        }

        row += 1;
    }

    if ctx.u8_at(AppContext::ALL_MAPS_OPEN)? != 0 {
        load_enemy_castle_csv(ctx, 2)?;
    }

    if redirect == 0 || ctx.u8_at(AppContext::EX_REDIRECT_ENABLED)? == 0 {
        return Ok(true);
    }

    if aku_realm_final_redirect(ctx, 0)? {
        if let Some(bytes) = open_asset_stream(ctx, b"MapStageDataRE_042.csv", 0, 0)? {
            let mut stm = AssetStream::new(&bytes, b'\n');
            let row_at = AppContext::MAP_STAGE_ROWS + 0x1d * 0xbc;

            read_csv_row(&mut stm);
            read_csv_row(&mut stm);
            read_csv_row(&mut stm);

            for col in 0..0x2eusize {
                let value = read_csv_cell(&stm, col as i32) as i32;
                let key = ctx.i32_at(row_at + 0xb8)?;

                ctx.set_i32_at(row_at + col * 4, value ^ key)?;

                if ctx.i32_at(row_at + col * 4)? ^ ctx.i32_at(row_at + 0xb8)? == -1 {
                    break;
                }
            }
        }

        return Ok(true);
    }

    if ex_redirect_check_c(ctx, 0)? {
        if let Some(bytes) = open_asset_stream(ctx, b"MapStageDataRE_068.csv", 0, 0)? {
            let mut stm = AssetStream::new(&bytes, b'\n');
            let row_at = AppContext::MAP_STAGE_ROWS + 5 * 0xbc;

            read_csv_row(&mut stm);
            read_csv_row(&mut stm);
            read_csv_row(&mut stm);

            for col in 0..0x2eusize {
                let value = read_csv_cell(&stm, col as i32) as i32;
                let key = ctx.i32_at(row_at + 0xb8)?;

                ctx.set_i32_at(row_at + col * 4, value ^ key)?;

                if ctx.i32_at(row_at + col * 4)? ^ ctx.i32_at(row_at + 0xb8)? == -1 {
                    break;
                }
            }
        }

        return Ok(true);
    }

    let map_id = get_global_map_id(ctx, 0)?;

    if !ex_replacement_pending(ctx, map_id, -1)? {
        return Ok(true);
    }

    let map_id = get_global_map_id(ctx, 0)?;
    let target = get_ex_option_target(ctx, map_id);
    let name = string_format_int(ctx, b"MapStageDataRE_%03d.csv", map_index_of_map_id(target))?;

    let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? else {
        return Ok(true);
    };
    let mut stm = AssetStream::new(&bytes, b'\n');

    read_csv_row(&mut stm);
    read_csv_row(&mut stm);

    let map_type = get_map_type(ctx, 0)?;
    let map_index = get_map_index(ctx, 0)?;
    let count = get_stage_count(ctx, map_type, map_index)?;

    if count <= 0 {
        return Ok(true);
    }

    for row in 0..count as u32 as usize {
        read_csv_row(&mut stm);

        let map_id = get_global_map_id(ctx, 0)?;

        if !ex_replacement_pending(ctx, map_id, row as i32)? {
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
    }

    Ok(true)
}
