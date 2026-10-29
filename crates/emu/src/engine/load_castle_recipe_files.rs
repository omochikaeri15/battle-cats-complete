use crate::Fault;

use super::{
    AppContext, AssetStream, get_column_count, load_castle_recipe_unlock_data_csv, open_asset_stream, read_cell_stream,
    read_csv_cell, read_csv_row, read_stream_row, reset_cannon_part_rows, string_format_int,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct CastleRecipeEntry {
    pub ids: Vec<i32>,
    pub values: Vec<i32>,
    pub rows: Vec<Vec<i32>>,
}

pub fn load_castle_recipe_files(ctx: &mut AppContext) -> Result<(), Fault> {
    load_castle_recipe_unlock_data_csv(ctx)?;

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

            ctx.castle_recipe_unlocks.entry(key).or_default().push(values);
        }
    }

    ctx.castle_recipes.clear();

    let mut loaded: Vec<i32> = Vec::new();
    let indices: Vec<i32> = ctx
        .castle_recipe_unlocks
        .iter()
        .flat_map(|(&index, rows)| rows.iter().map(move |_| index))
        .collect();

    for index in indices {
        if loaded.contains(&index) {
            continue;
        }

        loaded.push(index);

        for (pattern, kind) in [
            (b"CastleRecipe_%03d.csv".as_slice(), 0i32),
            (b"BaseRecipe_%03d.csv".as_slice(), 1),
            (b"DecoRecipe_%03d.csv".as_slice(), 2),
        ] {
            let name = string_format_int(ctx, pattern, index)?;

            let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? else {
                continue;
            };
            let stm = &mut AssetStream::new(&bytes, b'\n');

            *ctx.castle_recipes.entry(index).or_default().entry(kind).or_default() = CastleRecipeEntry::default();

            while read_csv_row(stm) {
                let id = read_csv_cell(stm, 0) as i32;

                ctx.castle_recipes.entry(index).or_default().entry(kind).or_default().ids.push(id);

                let value = read_csv_cell(stm, 1) as i32;

                ctx.castle_recipes.entry(index).or_default().entry(kind).or_default().values.push(value);

                let mut row: Vec<i32> = Vec::new();
                let mut column = 2i32;

                while column != 0x12 {
                    row.push(read_csv_cell(stm, column) as i32);
                    column += 1;
                }

                ctx.castle_recipes.entry(index).or_default().entry(kind).or_default().rows.push(row);
            }
        }
    }

    ctx.castle_mix_recipes.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"CastleMixRecipe.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            let key = read_csv_cell(stm, 0) as i32;
            let mix = [
                key,
                read_csv_cell(stm, 1) as i32,
                read_csv_cell(stm, 2) as i32,
                read_csv_cell(stm, 3) as i32,
                read_csv_cell(stm, 4) as i32,
            ];

            ctx.castle_mix_recipes.insert(key, mix);
        }
    }

    ctx.castle_recipe_texts.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"CastleRecipeDescriptions.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_stream_row(stm, b',') {
            let name = read_cell_stream(stm, 1).to_vec();
            let key = read_csv_cell(stm, 0) as i32;

            ctx.castle_recipe_names.insert(key, name);

            let mut column = 2i32;

            while column != 0x10 {
                let key = read_csv_cell(stm, 0) as i32;
                let line = read_cell_stream(stm, column).to_vec();

                ctx.castle_recipe_texts.entry(key).or_default().push(line);
                column += 1;
            }
        }
    }

    reset_cannon_part_rows(ctx);

    Ok(())
}
