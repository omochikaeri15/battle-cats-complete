use crate::Fault;

use super::{
    AppContext, AssetStream, get_column_count, open_asset_stream, read_cell_stream, read_csv_cell,
    read_csv_row, read_stream_row, string_format_int,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct CastleRecipeEntry {
    pub ids: Vec<i32>,
    pub values: Vec<i32>,
    pub rows: Vec<Vec<i32>>,
}

pub fn load_castle_recipe_files(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.castle_recipe_unlocks.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"CastleRecipeUnlock.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            let mut values: Vec<i32> = Vec::new();
            let mut column = 0i32;

            while column < get_column_count(stm) as i32 {
                values.push(read_csv_cell(stm, column) as i32);
                column += 1;
            }

            let key = read_csv_cell(stm, 0) as i32;

            ctx.castle_recipe_unlocks.insert(key, values);
        }
    }

    ctx.castle_mix_recipes.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"CastleMixRecipe.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            let key = read_csv_cell(stm, 0) as i32;
            let values = [
                read_csv_cell(stm, 1) as i32,
                read_csv_cell(stm, 2) as i32,
                read_csv_cell(stm, 3) as i32,
                read_csv_cell(stm, 4) as i32,
            ];

            ctx.castle_mix_recipes.insert(key, values);
        }
    }

    ctx.castle_recipe_texts.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"CastleRecipeDescriptions.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_stream_row(stm, b',') {
            let key = read_csv_cell(stm, 0) as i32;
            let mut lines: Vec<Vec<u8>> = Vec::new();
            let mut column = 1i32;

            while column != 0x10 {
                lines.push(read_cell_stream(stm, column).to_vec());
                column += 1;
            }

            ctx.castle_recipe_texts.insert(key, lines);
        }
    }

    ctx.castle_recipes.clear();

    let indices: Vec<i32> = ctx.castle_recipe_unlocks.keys().copied().collect();

    for index in indices {
        for (pattern, kind, first) in [
            (b"CastleRecipe_%03d.csv".as_slice(), 0i32, 1i32),
            (b"BaseRecipe_%03d.csv".as_slice(), 1, 1),
            (b"DecoRecipe_%03d.csv".as_slice(), 2, 2),
        ] {
            let name = string_format_int(ctx, pattern, index)?;

            let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? else {
                continue;
            };
            let stm = &mut AssetStream::new(&bytes, b'\n');

            while read_csv_row(stm) {
                let id = read_csv_cell(stm, 0) as i32;
                let value = if first == 2 {
                    Some(read_csv_cell(stm, 1) as i32)
                } else {
                    None
                };
                let mut row: Vec<i32> = Vec::new();
                let mut column = first;

                while column != 0x12 {
                    row.push(read_csv_cell(stm, column) as i32);
                    column += 1;
                }

                let entry = ctx
                    .castle_recipes
                    .entry(index)
                    .or_default()
                    .entry(kind)
                    .or_default();

                entry.ids.push(id);

                if let Some(value) = value {
                    entry.values.push(value);
                }

                entry.rows.push(row);
            }
        }
    }

    Ok(())
}
