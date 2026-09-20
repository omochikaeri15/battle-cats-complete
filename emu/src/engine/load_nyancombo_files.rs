use crate::Fault;

use super::{
    AppContext, AssetStream, format_localized, get_column_count, load_nyancombo_filter_tsv, load_nyancombo_param_tsv, open_asset_stream, query_localizable, read_cell_stream, read_csv_cell, read_csv_row, read_stream_row,
};

pub fn load_nyancombo_files(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.combo_names.clear();

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"Nyancombo_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_stream_row(stm, b',') {
            let cell = read_cell_stream(stm, 0).to_vec();

            ctx.combo_names.push(cell);
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"Nyancombo1_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.combo_effect_texts.clear();

        let mut slot = 0;

        while slot != 29 {
            read_stream_row(stm, b',');

            let cell = read_cell_stream(stm, 0).to_vec();

            ctx.combo_effect_texts.push(cell);
            slot += 1;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"Nyancombo2_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.combo_power_texts.clear();

        let mut slot = 0;

        while slot != 6 {
            read_stream_row(stm, b',');

            let cell = read_cell_stream(stm, 0).to_vec();

            ctx.combo_power_texts.push(cell);
            slot += 1;
        }
    }

    if let Some(bytes) = open_asset_stream(ctx, b"NyancomboData.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            if read_csv_cell(stm, 0) == -1 {
                break;
            }

            let mut values: Vec<i32> = Vec::new();
            let count = get_column_count(stm) as i32;
            let mut column = 0i32;

            while read_csv_cell(stm, 0) != -1 && column < count {
                if column >= 0xf && read_csv_cell(stm, column) == -1 {
                    break;
                }

                let value = read_csv_cell(stm, column) as i32;

                values.push(value);
                column += 1;
            }

            if column < count {
                let value = read_csv_cell(stm, column) as i32;

                values.push(value);
            }

            ctx.combo_definitions.push(values);
        }
    }

    load_nyancombo_param_tsv(ctx)?;
    load_nyancombo_filter_tsv(ctx)?;

    ctx.combo_store.states.clear();

    let mut definition = 0;

    while definition < ctx.combo_definitions.len() as i32 {
        ctx.combo_store.states.push(2);
        definition += 1;
    }

    Ok(())
}
