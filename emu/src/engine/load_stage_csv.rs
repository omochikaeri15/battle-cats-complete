use crate::{Fault, operation};

use super::{
    AppContext, AssetStream, STAGE_ENEMY_COLUMNS, altar_replacement, get_castle_enemy_row,
    get_map_type, get_stage_index, has_castle_enemy, labyrinth_stage_id, map_type_as_index,
    open_asset_stream, pack_entry_text, parse_stage_enemy_row, read_csv_cell, read_csv_row,
    set_scene, set_stage_entry_enemy, stage_enemy_valid, stage_entry_castle_defaults,
    stage_entry_enemy_id, stage_entry_row, stage_not_sealed, string_format_int, string_format_int2,
};

const LEGEND_FILES: [&[u8]; 5] = [
    b"stageRN%03d_%02d.csv",
    b"stageRS%03d_%02d.csv",
    b"stageRC%03d_%02d.csv",
    b"stageRT%03d_%02d.csv",
    b"stageRR%03d_%02d.csv",
];

const BATTLE_FILES: [(i32, &[u8]); 15] = [
    (-4, b"stageRV%03d_%02d.csv"),
    (-6, b"stageRM%03d_%02d.csv"),
    (-9, b"stageRNA%03d_%02d.csv"),
    (-10, b"stageRB%03d_%02d.csv"),
    (-11, b"stageRN%03d_%02d.csv"),
    (-16, b"stageRA%03d_%02d.csv"),
    (-17, b"stageRH%03d_%02d.csv"),
    (-18, b"stageRCA%03d_%02d.csv"),
    (-19, b"stageDM%03d_%02d.csv"),
    (-20, b"stageRQ%03d_%02d.csv"),
    (-21, b"stageL%03d_%02d.csv"),
    (-22, b"stageRND%03d_%02d.csv"),
    (-23, b"stageRSR%03d_%02d.csv"),
    (-24, b"stageG%03d_%02d.csv"),
    (-26, b"stageRPR%03d_%02d.csv"),
];

const PREVIEW_FILES: [(i32, &[u8]); 15] = [
    (-4, b"stageRV%03d_%02d.csv"),
    (-6, b"stageRM%03d_%02d.csv"),
    (-9, b"stageRNA%03d_%02d.csv"),
    (-8, b"stageEX%03d_%02d.csv"),
    (-10, b"stageRB%03d_%02d.csv"),
    (-16, b"stageRA%03d_%02d.csv"),
    (-17, b"stageRH%03d_%02d.csv"),
    (-18, b"stageRCA%03d_%02d.csv"),
    (-19, b"stageDM%03d_%02d.csv"),
    (-20, b"stageRQ%03d_%02d.csv"),
    (-21, b"stageL%03d_%02d.csv"),
    (-22, b"stageRND%03d_%02d.csv"),
    (-23, b"stageRSR%03d_%02d.csv"),
    (-24, b"stageG%03d_%02d.csv"),
    (-26, b"stageRPR%03d_%02d.csv"),
];

#[derive(PartialEq, Eq)]
enum Tail {
    Plain,
    Header,
}

pub fn load_stage_csv(ctx: &mut AppContext, stage: i32, check_pack: i32) -> Result<bool, Fault> {
    if check_pack == 0 {
        if ctx.i32_at(AppContext::CHAPTER_MODE)? != 3 {
            return Ok(true);
        }

        if ctx.i32_at(AppContext::SAVED_MAP_TYPE)? == map_type_as_index(-11) {
            return Ok(true);
        }

        let map_type = ctx.i32_at(AppContext::SAVED_MAP_TYPE)?;
        let mut picked: Option<(&[u8], i32)> = None;

        if (map_type as u32) <= 3 {
            picked = Some((LEGEND_FILES[map_type as usize], stage));
        } else {
            for (kind, pattern) in PREVIEW_FILES {
                if ctx.i32_at(AppContext::SAVED_MAP_TYPE)? == map_type_as_index(kind) {
                    let second = if kind == -21 {
                        labyrinth_stage_id(ctx, stage)?
                    } else {
                        stage
                    };

                    picked = Some((pattern, second));
                    break;
                }
            }
        }

        let bytes = match picked {
            Some((pattern, second)) => {
                let map = ctx.i32_at(AppContext::MAP_INDEX)?;
                let name = string_format_int2(ctx, pattern, map, second)?;

                open_asset_stream(ctx, &name, 0, 0)?.unwrap_or_default()
            }
            None => Vec::new(),
        };
        let mut stm = AssetStream::new(&bytes, b'\n');

        read_csv_row(&mut stm);

        let row = stage as i64 as usize;

        for col in 0..5usize {
            let value = read_csv_cell(&stm, col as i32 + 1) as i32;

            ctx.set_i32_at(
                AppContext::LEGEND_STAGE_INFO
                    .wrapping_add(row.wrapping_mul(0x14))
                    .wrapping_add(col * 4),
                value,
            )?;
        }

        read_csv_row(&mut stm);

        let energy = read_csv_cell(&stm, 6) as i32;

        if stage as u32 >= 0x64 {
            return Err(Fault::index_out_of_range(stage as i64, 0x64));
        }

        ctx.set_i32_at(
            AppContext::LEGEND_STAGE_ENERGY + row * 4,
            energy.wrapping_add(-2),
        )?;

        return Ok(true);
    }

    let key = ctx.data_pack_key.clone();
    let entry = pack_entry_text(ctx, &key)?;

    if entry != ctx.data_pack_expected {
        ctx.set_i32_at(AppContext::SCENE_4_PAGE, 0xb)?;
        set_scene(ctx, 4)?;

        return Ok(false);
    }

    let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;
    let (name, tail) = if mode == 0x63 {
        let ex_map = ctx.i32_at(AppContext::EX_MAP)?;
        let ex_stage = ctx.i32_at(AppContext::EX_STAGE)?;

        (
            Some(string_format_int2(
                ctx,
                b"stageEX%03d_%02d.csv",
                ex_map,
                ex_stage,
            )?),
            Tail::Plain,
        )
    } else {
        let flagged = mode == 3
            || ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0
            || ctx.u8_at(AppContext::BATTLE_IS_INVASION)? != 0
            || ctx.u8_at(AppContext::BATTLE_IS_Z_INVASION)? != 0;

        if flagged && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0 {
            let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;

            (
                Some(string_format_int2(
                    ctx,
                    b"stageZ%02d_%02d.csv",
                    mode,
                    stage,
                )?),
                Tail::Header,
            )
        } else if flagged && ctx.u8_at(AppContext::BATTLE_IS_INVASION)? != 0 {
            let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;

            (
                Some(string_format_int2(
                    ctx,
                    b"stageSpace%02d_Invasion_%02d.csv",
                    mode,
                    0,
                )?),
                Tail::Header,
            )
        } else if flagged && ctx.u8_at(AppContext::BATTLE_IS_Z_INVASION)? != 0 {
            let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;

            (
                Some(string_format_int2(
                    ctx,
                    b"stageSpace%02d_Invasion_Z_%02d.csv",
                    mode,
                    0,
                )?),
                Tail::Header,
            )
        } else if mode == 3 {
            let map_type = ctx.i32_at(AppContext::SAVED_MAP_TYPE)?;
            let mut picked: Option<(&[u8], i32, i32)> = None;

            if (map_type as u32) <= 4 {
                picked = Some((
                    LEGEND_FILES[map_type as usize],
                    ctx.i32_at(AppContext::MAP_INDEX)?,
                    stage,
                ));
            } else {
                for (kind, pattern) in BATTLE_FILES {
                    if ctx.i32_at(AppContext::SAVED_MAP_TYPE)? == map_type_as_index(kind) {
                        let pair = match kind {
                            -11 => {
                                let packed = ctx.i32_at(
                                    AppContext::DROP_MAP_STAGES
                                        .wrapping_add((stage as i64 as usize).wrapping_mul(4)),
                                )?;
                                let group = operation::div_100(packed);

                                (group, packed.wrapping_sub(group.wrapping_mul(100)))
                            }
                            -21 => {
                                let map = ctx.i32_at(AppContext::MAP_INDEX)?;

                                (map, labyrinth_stage_id(ctx, stage)?)
                            }
                            _ => (ctx.i32_at(AppContext::MAP_INDEX)?, stage),
                        };

                        picked = Some((pattern, pair.0, pair.1));
                        break;
                    }
                }
            }

            match picked {
                Some((pattern, first, second)) => (
                    Some(string_format_int2(ctx, pattern, first, second)?),
                    Tail::Header,
                ),
                None => (None, Tail::Header),
            }
        } else {
            let map_type = get_map_type(ctx, 0)?;

            if map_type == -2 {
                (
                    Some(string_format_int(ctx, b"stage%02d.csv", stage)?),
                    Tail::Plain,
                )
            } else if get_map_type(ctx, 0)? == -3 {
                let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;

                (
                    Some(string_format_int2(
                        ctx,
                        b"stageW%02d_%02d.csv",
                        mode,
                        stage,
                    )?),
                    Tail::Plain,
                )
            } else if get_map_type(ctx, 0)? == -7 {
                let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;

                (
                    Some(string_format_int2(
                        ctx,
                        b"stageSpace%02d_%02d.csv",
                        mode,
                        stage,
                    )?),
                    Tail::Plain,
                )
            } else {
                (None, Tail::Plain)
            }
        }
    };

    let bytes = match name {
        Some(name) => open_asset_stream(ctx, &name, 0, 0)?.unwrap_or_default(),
        None => Vec::new(),
    };
    let mut stm = AssetStream::new(&bytes, b'\n');

    read_csv_row(&mut stm);
    ctx.set_i32_at(AppContext::STAGE_CASTLE_ID, read_csv_cell(&stm, 0) as i32)?;
    ctx.set_i32_at(AppContext::STAGE_NO_CONTINUES, 0)?;
    ctx.set_i32_at(
        AppContext::STAGE_NO_CONTINUES,
        read_csv_cell(&stm, 1) as i32,
    )?;
    ctx.set_i32_at(AppContext::STAGE_EX_CHANCE, 0)?;
    ctx.set_i32_at(AppContext::STAGE_EX_CHANCE, read_csv_cell(&stm, 2) as i32)?;
    ctx.set_i32_at(AppContext::STAGE_EX_MAP, 0)?;
    ctx.set_i32_at(AppContext::STAGE_EX_MAP, read_csv_cell(&stm, 3) as i32)?;
    ctx.set_i32_at(AppContext::STAGE_EX_STAGE_MIN, 0)?;
    ctx.set_i32_at(
        AppContext::STAGE_EX_STAGE_MIN,
        read_csv_cell(&stm, 4) as i32,
    )?;
    ctx.set_i32_at(AppContext::STAGE_EX_STAGE_MAX, 0)?;
    ctx.set_i32_at(
        AppContext::STAGE_EX_STAGE_MAX,
        read_csv_cell(&stm, 5) as i32,
    )?;

    if tail == Tail::Header && ctx.i32_at(AppContext::SAVED_MAP_TYPE)? == map_type_as_index(-11) {
        let stage = get_stage_index(ctx)?;
        let row =
            ctx.play_dungeon_rows
                .get(stage as i64 as usize)
                .ok_or(Fault::index_out_of_range(stage as i64, ctx.play_dungeon_rows.len() as i64))?;

        ctx.set_i32_at(AppContext::STAGE_NO_CONTINUES, row[8] as u8 as i32)?;
    }

    read_csv_row(&mut stm);

    for col in 0..9usize {
        ctx.set_i32_at(
            AppContext::STAGE_LENGTH + col * 4,
            read_csv_cell(&stm, col as i32) as i32,
        )?;
    }

    ctx.stage_enemies.clear();

    while read_csv_row(&mut stm) {
        let mut entry = [0i32; STAGE_ENEMY_COLUMNS];

        parse_stage_enemy_row(&mut entry, &stm);

        if !stage_enemy_valid(&entry) {
            break;
        }

        ctx.stage_enemies.push(entry);
    }

    let length = ctx.i32_at(AppContext::STAGE_LENGTH)?;

    ctx.set_i32_at(AppContext::STAGE_LENGTH, length.wrapping_shl(2))?;

    if tail == Tail::Plain {
        return Ok(true);
    }

    if !has_castle_enemy(ctx)? || ctx.stage_enemies.is_empty() {
        return Ok(true);
    }

    let mut entry = 0usize;

    while entry < ctx.stage_enemies.len() {
        let row = stage_entry_row(&ctx.stage_enemies[entry]);

        if row == get_castle_enemy_row(ctx)? {
            stage_entry_castle_defaults(&mut ctx.stage_enemies[entry]);

            let enemy = stage_entry_enemy_id(&ctx.stage_enemies[entry]);

            if stage_not_sealed(ctx, enemy)? {
                let enemy = stage_entry_enemy_id(&ctx.stage_enemies[entry]);
                let replacement = altar_replacement(ctx, enemy);

                if replacement != -1 {
                    ctx.set_i32_at(AppContext::STAGE_LENGTH + 0x18, replacement.wrapping_add(2))?;
                    set_stage_entry_enemy(&mut ctx.stage_enemies[entry], replacement);
                }
            }
        }

        entry += 1;
    }

    Ok(true)
}
