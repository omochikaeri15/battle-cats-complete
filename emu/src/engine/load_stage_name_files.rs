use crate::Fault;

use super::{
    AppContext, AssetStream, format_localized, get_column_count, open_asset_stream,
    query_localizable, read_cell_stream, read_csv_cell, read_csv_cell_float, read_stream_row,
    read_tsv_row, std_string_equals, string_format_rank_comment,
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
        }

        if let Some(count) = ctx.stage_name_counts.get_mut(variant) {
            *count = ctx.stage_name_variants.get(variant).map_or(0, |rows| rows.len()) as i32;
        }
    }

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

        ctx.treasure2_texts.clear();

        let mut line = 0i32;

        while line != 12 {
            read_stream_row(stm, b',');
            ctx.treasure2_texts
                .push(read_cell_stream(stm, 0).to_vec());
            line += 1;
        }
    }

    for (pattern, after) in [
        (b"Treasure3_0_%@.csv".as_slice(), false),
        (
            b"Treasure3_1_AfterFirstEncounter_%@.csv".as_slice(),
            true,
        ),
    ] {
        let lang = query_localizable(ctx, b"lang");
        let name = format_localized(ctx, pattern, &lang)?;

        let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? else {
            continue;
        };
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let rows = if after {
            &mut ctx.treasure3_after_texts
        } else {
            &mut ctx.treasure3_texts
        };

        rows.clear();

        let mut line = 0i32;

        while line != 22 {
            read_stream_row(stm, b',');

            let entry = [
                read_cell_stream(stm, 0).to_vec(),
                read_cell_stream(stm, 1).to_vec(),
                read_cell_stream(stm, 2).to_vec(),
            ];
            let rows = if after {
                &mut ctx.treasure3_after_texts
            } else {
                &mut ctx.treasure3_texts
            };

            rows.push(entry);
            line += 1;
        }
    }

    Ok(())
}
