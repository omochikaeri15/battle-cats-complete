use std::mem;

use crate::Fault;

use super::{
    AppContext, AssetStream, format_localized, get_column_count, get_map_count,
    load_play_stage_limit_csv, open_asset_stream, query_localizable, read_cell_stream,
    read_csv_cell, read_csv_cell_float, read_stream_row, read_tsv_row, reset_stage_name_tables,
    stage_name_table_load, stage_name_table_size, std_string_equals, string_format_rank_comment,
};

const TERMINATOR: &[u8] = b"@";

pub fn load_stage_name_files(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.map_stage_limit_messages.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"MapStageLimitMessage.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_stream_row(stm, b',') {
            let key = read_csv_cell(stm, 0) as i32;
            let mut texts: [Vec<u8>; 3] = Default::default();
            let mut slot = 0i32;

            while slot != 3 {
                if let Some(text) = texts.get_mut(slot as usize) {
                    *text = read_cell_stream(stm, slot).to_vec();
                }

                slot += 1;
            }

            ctx.map_stage_limit_messages.insert(key, texts);
        }
    }

    let mut file = 0usize;

    while file != 3 {
        let lang = query_localizable(ctx, b"lang");
        let name =
            string_format_rank_comment(ctx, b"StageName%d_%@.csv", file as i32, &lang)?;

        if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
            let stm = &mut AssetStream::new(&bytes, b'\n');

            if let Some(slots) = ctx.stage_names_numbered.get_mut(file) {
                slots.clear();
            }

            let mut slot = 0i32;

            while slot != 49 {
                read_stream_row(stm, b',');

                let text = read_cell_stream(stm, 0).to_vec();

                if let Some(slots) = ctx.stage_names_numbered.get_mut(file) {
                    slots.push(text);
                }

                slot += 1;
            }
        }

        file += 1;
    }

    for (pattern, variant) in [
        (b"StageName_RN_%@.csv".as_slice(), 0usize),
        (b"StageName_RS_%@.csv".as_slice(), 1),
        (b"StageName_RC_%@.csv".as_slice(), 2),
        (b"StageName_RE_%@.csv".as_slice(), 3),
        (b"StageName_RT_%@.csv".as_slice(), 4),
        (b"StageName_RR_%@.csv".as_slice(), 5),
        (b"StageName_RV_%@.csv".as_slice(), 6),
        (b"StageName_RM_%@.csv".as_slice(), 7),
        (b"StageName_RNA_%@.csv".as_slice(), 8),
        (b"StageName_RB_%@.csv".as_slice(), 9),
        (b"StageName_RD_%@.csv".as_slice(), 10),
        (b"StageName_RA_%@.csv".as_slice(), 11),
        (b"StageName_RH_%@.csv".as_slice(), 12),
        (b"StageName_RCA_%@.csv".as_slice(), 13),
        (b"StageName_DM_%@.csv".as_slice(), 14),
        (b"StageName_RQ_%@.csv".as_slice(), 15),
        (b"StageName_L_%@.csv".as_slice(), 16),
    ] {
        if let Some(rows) = ctx.stage_name_variants.get_mut(variant) {
            rows.clear();
        }

        let lang = query_localizable(ctx, b"lang");
        let name = format_localized(ctx, pattern, &lang)?;

        if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
            let stm = &mut AssetStream::new(&bytes, b'\n');

            while read_stream_row(stm, b',') {
                if get_column_count(stm) == 0 {
                    break;
                }

                let mut column = 0i32;

                while column < get_column_count(stm) as i32 {
                    let text = read_cell_stream(stm, column).to_vec();

                    if std_string_equals(&text, TERMINATOR) {
                        break;
                    }

                    if let Some(rows) = ctx.stage_name_variants.get_mut(variant) {
                        if column == 0 {
                            rows.push(Vec::new());
                        }

                        if let Some(row) = rows.last_mut() {
                            row.push(text);
                        }
                    }

                    column += 1;
                }
            }

            if variant < 5 {
                let maps = ((ctx.stage_name_variants.get(variant).map_or(0, |rows| rows.len()) * 3) as u32)
                    .wrapping_mul(0xaaaa_aaab) as i32;

                ctx.set_i32_at(AppContext::STORY_MAP_COUNTS + variant * 4, maps)?;
            }
        }
    }

    load_play_stage_limit_csv(ctx)?;

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"StageName_RND_%@.csv", &lang)?;
    let mut table = mem::take(&mut ctx.stage_table_neg22);

    stage_name_table_load(ctx, &mut table, &name, b',')?;
    ctx.stage_table_neg22 = table;
    stage_name_table_size(&mut ctx.stage_table_neg22, 4, 0xc);

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"StageName_RSR_%@.csv", &lang)?;
    let mut table = mem::take(&mut ctx.stage_table_neg23);

    stage_name_table_load(ctx, &mut table, &name, b',')?;
    ctx.stage_table_neg23 = table;
    stage_name_table_size(&mut ctx.stage_table_neg23, 4, 0xc);

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"StageName_G_%@.csv", &lang)?;
    let mut table = mem::take(&mut ctx.stage_table_neg24);

    stage_name_table_load(ctx, &mut table, &name, b',')?;
    ctx.stage_table_neg24 = table;
    stage_name_table_size(&mut ctx.stage_table_neg24, 4, 0xc);

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"StageName_RPR_%@.csv", &lang)?;
    let mut table = mem::take(&mut ctx.stage_table_neg26);

    stage_name_table_load(ctx, &mut table, &name, b',')?;
    ctx.stage_table_neg26 = table;
    stage_name_table_size(&mut ctx.stage_table_neg26, 4, 0xc);

    ctx.stage_difficulty.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"difficulty_level.tsv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_tsv_row(stm) {
            if (get_column_count(stm) as i32) < 2 {
                continue;
            }

            let key = read_csv_cell(stm, 0) as i32;
            let mut levels: Vec<f32> = Vec::new();
            let mut column = 1i32;

            while column < get_column_count(stm) as i32 {
                levels.push(read_csv_cell_float(stm, column));
                column += 1;
            }

            ctx.stage_difficulty.insert(key, levels);
        }
    }

    let maps = ctx.stage_name_variants[3].len();

    ctx.stage_unlock_neg4.resize(maps * 4, 0);
    ctx.stages_cleared_neg4.resize(maps * 4, 0);
    ctx.stage_record_neg4.resize(maps * 200, 0);
    ctx.map_open_neg4.resize(maps * 4, 0);
    ctx.map_flag_neg4.resize(maps, 0);
    ctx.map_count_neg4.resize(maps, 0);
    ctx.map_stamp_neg4.resize(maps, 0);
    ctx.map_opened_neg4.resize(maps, false);

    let maps = ctx.stage_name_variants[8].len();

    ctx.stage_unlock_neg9.resize(maps * 4, 0);
    ctx.stages_cleared_neg9.resize(maps * 4, 0);
    ctx.stage_record_neg9.resize(maps * 48, 0);
    ctx.map_open_neg9.resize(maps * 4, 0);
    ctx.map_flag_neg9.resize(maps, 0);

    let maps = ctx.stage_name_variants[9].len();

    ctx.stage_unlock_neg10.resize(maps * 4, 0);
    ctx.stages_cleared_neg10.resize(maps * 4, 0);
    ctx.stage_record_neg10.resize(maps * 60, 0);
    ctx.map_open_neg10.resize(maps * 4, 0);
    ctx.map_flag_neg10.resize(maps, 0);
    ctx.map_count_neg10.resize(maps, 0);
    ctx.map_stamp_neg10.resize(maps, 0);
    ctx.map_opened_neg10.resize(maps, false);

    let maps = ctx.stage_name_variants[10].len();

    ctx.stage_unlock_neg11.resize(maps * 4, 0);
    ctx.stages_cleared_neg11.resize(maps * 4, 0);
    ctx.stage_record_neg11.resize(maps * 0xc0, 0);
    ctx.dungeon_clear_counts.resize(maps, [[0; 4]; 0x30]);
    ctx.map_open_neg11.resize(maps * 4, 0);
    ctx.map_flag_neg11.resize(maps, 0);
    ctx.map_count_neg11.resize(maps, 0);
    ctx.map_stamp_neg11.resize(maps, 0);
    ctx.map_opened_neg11.resize(maps, false);

    let maps = ctx.stage_name_variants[11].len();

    ctx.stage_unlock_neg16.resize(maps * 4, 0);
    ctx.stages_cleared_neg16.resize(maps * 4, 0);
    ctx.stage_record_neg16.resize(maps * 0x78, 0);
    ctx.map_open_neg16.resize(maps * 4, 0);
    ctx.map_flag_neg16.resize(maps, 0);
    ctx.map_count_neg16.resize(maps, 0);
    ctx.map_stamp_neg16.resize(maps, 0);
    ctx.map_opened_neg16.resize(maps, false);

    let maps = ctx.stage_name_variants[12].len();

    ctx.stage_unlock_neg17.resize(maps, 0);
    ctx.stages_cleared_neg17.resize(maps, 0);
    ctx.stage_record_neg17.resize(maps * 8, 0);
    ctx.map_open_neg17.resize(maps, 0);
    ctx.map_flag_neg17.resize(maps, 0);
    ctx.map_count_neg17.resize(maps, 0);
    ctx.map_stamp_neg17.resize(maps, 0);
    ctx.map_opened_neg17.resize(maps, false);

    let maps = ctx.stage_name_variants[13].len();

    ctx.stage_unlock_neg18.resize(maps * 4, 0);
    ctx.stages_cleared_neg18.resize(maps * 4, 0);
    ctx.stage_record_neg18.resize(maps * 0x78, 0);
    ctx.map_open_neg18.resize(maps * 4, 0);
    ctx.map_flag_neg18.resize(maps, 0);
    ctx.map_count_neg18.resize(maps, 0);
    ctx.map_stamp_neg18.resize(maps, 0);
    ctx.map_opened_neg18.resize(maps, false);

    let maps = ctx.stage_name_variants[15].len();

    ctx.stage_unlock_neg20.resize(maps * 4, 0);
    ctx.stages_cleared_neg20.resize(maps * 4, 0);
    ctx.stage_record_neg20.resize(maps * 0x78, 0);
    ctx.map_open_neg20.resize(maps * 4, 0);
    ctx.map_flag_neg20.resize(maps, 0);
    ctx.map_count_neg20.resize(maps, 0);
    ctx.map_stamp_neg20.resize(maps, 0);
    ctx.map_opened_neg20.resize(maps, false);

    let maps = get_map_count(ctx, -0x15)? as i64 as usize;

    ctx.map_count_neg21.resize(maps, 0);

    let maps = get_map_count(ctx, -0x15)? as i64 as usize;

    ctx.map_stamp_neg21.resize(maps, 0);

    let maps = get_map_count(ctx, -0x15)? as i64 as usize;

    ctx.map_opened_neg21.resize(maps, false);

    let maps = ctx.stage_name_variants[14].len();

    ctx.stage_unlock_neg19.resize(maps, 0);
    ctx.stage_record_neg19.resize(maps * 49, 0);

    let maps = get_map_count(ctx, 3)? as i64 as usize;

    ctx.map_stamp_3.resize(maps, 0);

    let maps = get_map_count(ctx, 3)? as i64 as usize;

    ctx.map_opened_3.resize(maps, false);

    let maps = get_map_count(ctx, 4)? as i64 as usize;

    ctx.map_stamp_4.resize(maps, 0);

    reset_stage_name_tables(ctx)?;

    let mut slot = 0usize;

    while slot != 10 {
        let lang = query_localizable(ctx, b"lang");
        let name = if slot <= 2 {
            Some(format_localized(ctx, b"Treasure1_0_%@.csv", &lang)?)
        } else if slot == 3 {
            None
        } else if slot <= 6 {
            Some(format_localized(ctx, b"Treasure1_1_%@.csv", &lang)?)
        } else {
            Some(string_format_rank_comment(
                ctx,
                b"Treasure1_2_%d_%@.csv",
                slot.wrapping_sub(7) as i32,
                &lang,
            )?)
        };

        if let Some(name) = name
            && let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)?
        {
            let stm = &mut AssetStream::new(&bytes, b'\n');

            if let Some(texts) = ctx.treasure1_texts.get_mut(slot) {
                texts.clear();
            }

            let mut line = 0i32;

            while line != 49 {
                read_stream_row(stm, b',');

                let text = read_cell_stream(stm, 0).to_vec();

                if let Some(texts) = ctx.treasure1_texts.get_mut(slot) {
                    texts.push(text);
                }

                line += 1;
            }
        }

        slot += 1;
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"Treasure2_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        let mut line = 0usize;

        while line != 12 {
            read_stream_row(stm, b',');

            let text = read_cell_stream(stm, 0).to_vec();

            if let Some(slot) = ctx.treasure2_texts.get_mut(line + 1) {
                *slot = text;
            }

            line += 1;
        }
    }

    let mut table = 0usize;

    while table != 10 {
        let name = if table <= 2 {
            let lang = query_localizable(ctx, b"lang");

            Some(format_localized(ctx, b"Treasure3_0_%@.csv", &lang)?)
        } else if table == 3 {
            None
        } else if table <= 6 {
            let lang = query_localizable(ctx, b"lang");

            Some(format_localized(ctx, b"Treasure3_1_%@.csv", &lang)?)
        } else {
            let lang = query_localizable(ctx, b"lang");

            Some(string_format_rank_comment(
                ctx,
                b"Treasure3_2_%d_%@.csv",
                table.wrapping_sub(7) as i32,
                &lang,
            )?)
        };

        if let Some(name) = name
            && let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)?
        {
            let stm = &mut AssetStream::new(&bytes, b'\n');
            let mut row = 0usize;

            while row != 23 {
                read_stream_row(stm, b',');

                let entry = [
                    read_cell_stream(stm, 0).to_vec(),
                    read_cell_stream(stm, 1).to_vec(),
                    read_cell_stream(stm, 2).to_vec(),
                ];

                if let Some(slot) = ctx.treasure3_texts.get_mut(table).and_then(|rows| rows.get_mut(row)) {
                    *slot = entry;
                }

                row += 1;
            }
        }

        table += 1;
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"Treasure3_1_AfterFirstEncounter_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut row = 0usize;

        while row != 23 {
            read_stream_row(stm, b',');

            let entry = [
                read_cell_stream(stm, 0).to_vec(),
                read_cell_stream(stm, 1).to_vec(),
                read_cell_stream(stm, 2).to_vec(),
            ];

            if let Some(slot) = ctx.treasure3_after_texts.get_mut(row) {
                *slot = entry;
            }

            row += 1;
        }
    }

    Ok(())
}
