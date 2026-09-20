use crate::Fault;

use super::{
    AppContext, AssetStream, format_localized, open_asset_stream, query_localizable,
    read_cell_stream, read_csv_cell, read_csv_row, read_stream_row, std_string_equals,
    string_format_rank_comment,
};

const TERMINATOR: &[u8] = &[0xef, 0xbc, 0xa0];

pub fn load_gamatoto_files(ctx: &mut AppContext) -> Result<(), Fault> {
    let mut log = 0usize;

    while log != 3 {
        let lang = query_localizable(ctx, b"lang");
        let name = string_format_rank_comment(
            ctx,
            b"GamatotoExpedition_Log_%d_%@.csv",
            log.wrapping_add(1) as i32,
            &lang,
        )?;

        if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
            let stm = &mut AssetStream::new(&bytes, b'\n');
            let mut row = 0i32;

            if let Some(lines) = ctx.gamatoto_logs.get_mut(log) {
                lines.clear();
            }

            while row != 0x200 {
                if !read_stream_row(stm, b',') {
                    break;
                }

                let text = read_cell_stream(stm, 0).to_vec();
                let stop = std_string_equals(&text, TERMINATOR);

                if let Some(lines) = ctx.gamatoto_logs.get_mut(log) {
                    lines.push(text);
                }

                if stop {
                    break;
                }

                row += 1;
            }

            if let Some(count) = ctx.gamatoto_log_counts.get_mut(log) {
                *count = row;
            }
        }

        log += 1;
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"GamatotoExpedition_Members_name_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut row = 0i32;

        ctx.gamatoto_member_stats.clear();
        ctx.gamatoto_member_names.clear();

        while row != 0x101 {
            if !read_stream_row(stm, b',') {
                break;
            }

            ctx.gamatoto_member_stats.push([
                read_csv_cell(stm, 0) as i32,
                read_csv_cell(stm, 1) as i32,
                read_csv_cell(stm, 2) as i32,
            ]);

            let mut names: [Vec<u8>; 4] = Default::default();
            let mut column = 3i32;

            loop {
                if let Some(slot) = names.get_mut(column.wrapping_sub(3) as usize) {
                    *slot = read_cell_stream(stm, column).to_vec();
                }

                if column >= 5 {
                    break;
                }

                column += 1;
            }

            ctx.gamatoto_member_names.push(names);
            row += 1;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"GamatotoExpedition_Message_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut row = 0i32;

        ctx.gamatoto_messages.clear();

        while row != 0x2a {
            if !read_stream_row(stm, b',') {
                break;
            }

            let mut lines: [Vec<u8>; 5] = Default::default();
            let mut column = 0i32;

            loop {
                let text = read_cell_stream(stm, column).to_vec();
                let stop = std_string_equals(&text, TERMINATOR);

                if let Some(slot) = lines.get_mut(column as usize) {
                    *slot = text;
                }

                if column >= 4 || stop {
                    break;
                }

                column += 1;
            }

            ctx.gamatoto_messages.push(lines);
            row += 1;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"GamatotoExpedition_Stage_Free_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut row = 0i32;

        ctx.gamatoto_stage_free.clear();

        while row != 0x40 {
            if !read_stream_row(stm, b',') {
                break;
            }

            ctx.gamatoto_stage_free.push([
                read_cell_stream(stm, 0).to_vec(),
                read_cell_stream(stm, 1).to_vec(),
                read_cell_stream(stm, 2).to_vec(),
            ]);
            row += 1;
        }
    }

    for (pattern, count, event) in [
        (
            b"GamatotoExpedition_Stage_name_%@.csv".as_slice(),
            0x40i32,
            false,
        ),
        (
            b"GamatotoExpedition_Stage_nameEvent_%@.csv".as_slice(),
            0x10,
            true,
        ),
    ] {
        let lang = query_localizable(ctx, b"lang");
        let name = format_localized(ctx, pattern, &lang)?;

        let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? else {
            continue;
        };
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut row = 0i32;

        if event {
            ctx.gamatoto_stage_event_names.clear();
        } else {
            ctx.gamatoto_stage_names.clear();
        }

        while row != count {
            if !read_stream_row(stm, b',') {
                break;
            }

            let text = read_cell_stream(stm, 0).to_vec();

            if event {
                ctx.gamatoto_stage_event_names.push(text);
            } else {
                ctx.gamatoto_stage_names.push(text);
            }

            row += 1;
        }
    }

    if let Some(bytes) = open_asset_stream(ctx, b"GamatotoExpedition_Limit.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_csv_row(stm);
        ctx.gamatoto_limit = [
            read_csv_cell(stm, 0) as i32,
            read_csv_cell(stm, 1) as i32,
            read_csv_cell(stm, 2) as i32,
        ];
    }

    for (file, count, event) in [
        (b"GamatotoExpedition_Stage.csv".as_slice(), 0x40i32, false),
        (b"GamatotoExpedition_Stage_EVENT.csv".as_slice(), 0x10, true),
    ] {
        let Some(bytes) = open_asset_stream(ctx, file, 0, 0)? else {
            continue;
        };
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let rows = if event {
            &mut ctx.gamatoto_stage_event_rows
        } else {
            &mut ctx.gamatoto_stage_rows
        };

        *rows = vec![[-1i32; 45]; count as usize];

        let mut row = 0usize;

        while row != count as usize {
            if !read_csv_row(stm) {
                break;
            }

            let mut column = 0i32;

            while column != 0x2d {
                let value = read_csv_cell(stm, column) as i32;
                let rows = if event {
                    &mut ctx.gamatoto_stage_event_rows
                } else {
                    &mut ctx.gamatoto_stage_rows
                };

                if let Some(slot) = rows.get_mut(row).and_then(|r| r.get_mut(column as usize)) {
                    *slot = value;
                }

                column += 1;
            }

            row += 1;
        }
    }

    if let Some(bytes) =
        open_asset_stream(ctx, b"GamatotoExpedition_SpecialDropItem.csv", 0, 0)?
    {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.gamatoto_drop_rows = vec![[-1i32; 4]; 0x82];

        let mut row = 0usize;

        while row != 0x82 {
            if !read_csv_row(stm) {
                break;
            }

            let values = [
                read_csv_cell(stm, 0) as i32,
                read_csv_cell(stm, 1) as i32,
                read_csv_cell(stm, 2) as i32,
                read_csv_cell(stm, 3) as i32,
            ];

            if let Some(slot) = ctx.gamatoto_drop_rows.get_mut(row) {
                *slot = values;
            }

            row += 1;
        }
    }

    if let Some(bytes) = open_asset_stream(ctx, b"GamatotoExpedition_Members.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.gamatoto_member_ids = [-1i32; 71];

        let mut row = 0usize;

        while row != 71 {
            let value = if read_csv_row(stm) {
                read_csv_cell(stm, 0) as i32
            } else {
                -1
            };

            if let Some(slot) = ctx.gamatoto_member_ids.get_mut(row) {
                *slot = value;
            }

            row += 1;
        }
    }

    Ok(())
}
