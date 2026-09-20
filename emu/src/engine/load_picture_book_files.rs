use crate::Fault;

use super::{
    AppContext, AssetStream, format_localized, get_column_count, open_asset_stream,
    query_localizable, read_cell_stream, read_csv_cell, read_csv_row, read_stream_row, read_tsv_row,
};

const SITE: &str = "load_picture_book_files";

pub fn load_picture_book_files(ctx: &mut AppContext) -> Result<(), Fault> {
    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"EnemyPictureBook_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut row = 0i32;

        ctx.enemy_book_rows.clear();

        while row != 800 {
            read_stream_row(stm, b',');

            ctx.enemy_book_rows.push([
                read_cell_stream(stm, 0).to_vec(),
                read_cell_stream(stm, 1).to_vec(),
                read_cell_stream(stm, 2).to_vec(),
                read_cell_stream(stm, 3).to_vec(),
                read_cell_stream(stm, 4).to_vec(),
            ]);
            row += 1;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"EnemyPictureBook2_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut row = 0i32;

        ctx.enemy_book_pages.clear();

        while row != 0x322 {
            read_stream_row(stm, b',');

            let mut page: [Vec<u8>; 12] = Default::default();
            let mut column = 0i32;

            while column != 0xc {
                if let Some(slot) = page.get_mut(column as usize) {
                    *slot = read_cell_stream(stm, column).to_vec();
                }

                column += 1;
            }

            ctx.enemy_book_pages.push(page);
            row += 1;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"EnemyPictureBookQuestion_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_stream_row(stm, b',');

        ctx.enemy_book_question = [
            read_cell_stream(stm, 0).to_vec(),
            read_cell_stream(stm, 1).to_vec(),
            read_cell_stream(stm, 2).to_vec(),
            read_cell_stream(stm, 3).to_vec(),
            read_cell_stream(stm, 4).to_vec(),
        ];
    }

    if let Some(bytes) = open_asset_stream(ctx, b"Enemyname.tsv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.enemy_names.clear();

        while read_tsv_row(stm) {
            ctx.enemy_names.push(read_cell_stream(stm, 0).to_vec());
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"nyankoPictureBook_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut row = 0i32;

        ctx.cat_book_pages.clear();

        while row != 0x36c {
            read_stream_row(stm, b',');

            let mut page: [Vec<u8>; 12] = Default::default();
            let mut column = 0i32;

            while column != 0xc {
                if let Some(slot) = page.get_mut(column as usize) {
                    *slot = read_cell_stream(stm, column).to_vec();
                }

                column += 1;
            }

            ctx.cat_book_pages.push(page);
            row += 1;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"nyankoPictureBookQuestion_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_stream_row(stm, b',');

        ctx.cat_book_question = [
            read_cell_stream(stm, 0).to_vec(),
            read_cell_stream(stm, 1).to_vec(),
            read_cell_stream(stm, 2).to_vec(),
        ];
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"nyankoPictureBook2_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.cat_book_rows.clear();

        while read_stream_row(stm, b',') {
            ctx.cat_book_rows.push([
                read_cell_stream(stm, 0).to_vec(),
                read_cell_stream(stm, 1).to_vec(),
                read_cell_stream(stm, 2).to_vec(),
                read_cell_stream(stm, 3).to_vec(),
                read_cell_stream(stm, 4).to_vec(),
            ]);
        }
    }

    if let Some(bytes) = open_asset_stream(ctx, b"nyankoPictureBookData.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut row = 0i32;

        ctx.cat_book_data.clear();

        while row != 0x36c {
            read_csv_row(stm);

            ctx.cat_book_data.push([
                read_csv_cell(stm, 0) as i32,
                read_csv_cell(stm, 1) as i32,
                read_csv_cell(stm, 2) as i32,
                read_csv_cell(stm, 3) as i32,
                read_csv_cell(stm, 4) as i32,
                read_csv_cell(stm, 5) as i32,
                read_csv_cell(stm, 6) as i32,
                read_csv_cell(stm, 7) as i32,
            ]);
            row += 1;
        }
    }

    if let Some(bytes) = open_asset_stream(ctx, b"nyankoPictureBookData_EffectAbility.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.picture_book_abilities.clear();

        while read_csv_row(stm) {
            if read_csv_cell(stm, 0) == -1 {
                break;
            }

            ctx.picture_book_abilities.push(vec![
                read_csv_cell(stm, 1) as i32,
                read_csv_cell(stm, 2) as i32,
                read_csv_cell(stm, 3) as i32,
                read_csv_cell(stm, 4) as i32,
                read_csv_cell(stm, 5) as i32,
                read_csv_cell(stm, 6) as i32,
            ]);
        }
    }

    if let Some(bytes) = open_asset_stream(ctx, b"nyankoPictureBookData_Attribute.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.picture_book_traits.clear();

        while read_csv_row(stm) {
            if read_csv_cell(stm, 0) == -1 {
                break;
            }

            ctx.picture_book_traits.push(vec![
                read_csv_cell(stm, 1) as i32,
                read_csv_cell(stm, 2) as i32,
                read_csv_cell(stm, 3) as i32,
                read_csv_cell(stm, 4) as i32,
            ]);
        }

        let mut order = [0i32, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut sorted = 1usize;

        while sorted != 10 {
            let right = order.get(sorted).copied().unwrap_or(0);
            let left = order.get(sorted.wrapping_sub(1)).copied().unwrap_or(0);
            let right_key = ctx
                .picture_book_traits
                .get(right as usize)
                .and_then(|row| row.first())
                .copied()
                .ok_or(Fault::IndexOutOfRange {
                    site: SITE,
                    index: right as i64,
                    limit: ctx.picture_book_traits.len() as i64,
                })?;
            let left_key = ctx
                .picture_book_traits
                .get(left as usize)
                .and_then(|row| row.first())
                .copied()
                .ok_or(Fault::IndexOutOfRange {
                    site: SITE,
                    index: left as i64,
                    limit: ctx.picture_book_traits.len() as i64,
                })?;

            if right_key >= left_key {
                sorted += 1;

                continue;
            }

            let key = right;
            let mut slot = sorted;

            loop {
                let moved = order.get(slot.wrapping_sub(1)).copied().unwrap_or(0);

                if let Some(cell) = order.get_mut(slot) {
                    *cell = moved;
                }

                if slot == 1 {
                    break;
                }

                let below = order.get(slot.wrapping_sub(2)).copied().unwrap_or(0);
                let below_key = ctx
                    .picture_book_traits
                    .get(below as usize)
                    .and_then(|row| row.first())
                    .copied()
                    .ok_or(Fault::IndexOutOfRange {
                        site: SITE,
                        index: below as i64,
                        limit: ctx.picture_book_traits.len() as i64,
                    })?;
                let key_value = ctx
                    .picture_book_traits
                    .get(key as usize)
                    .and_then(|row| row.first())
                    .copied()
                    .ok_or(Fault::IndexOutOfRange {
                        site: SITE,
                        index: key as i64,
                        limit: ctx.picture_book_traits.len() as i64,
                    })?;

                slot -= 1;

                if key_value >= below_key {
                    break;
                }
            }

            if let Some(cell) = order.get_mut(slot) {
                *cell = key;
            }

            sorted += 1;
        }

        ctx.picture_book_trait_order.clear();

        for index in order {
            ctx.picture_book_trait_order.push(index);
        }
    }

    if let Some(bytes) = open_asset_stream(ctx, b"enemy_dictionary_list.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.enemy_dictionary_ids.clear();
        ctx.enemy_dictionary_pages.clear();
        ctx.enemy_dictionary_groups.clear();

        while read_csv_row(stm) {
            ctx.enemy_dictionary_ids
                .push((read_csv_cell(stm, 0) as i32).wrapping_add(2));
            ctx.enemy_dictionary_pages
                .push(read_csv_cell(stm, 1) as i32);

            if get_column_count(stm) as i32 > 2 {
                ctx.enemy_dictionary_groups
                    .push(read_csv_cell(stm, 2) as i32);
            } else {
                ctx.enemy_dictionary_groups.push(-1);
            }
        }
    }

    Ok(())
}
