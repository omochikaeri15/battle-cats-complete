use crate::Fault;

use super::{
    AppContext, AssetStream, format_localized, open_asset_stream, query_localizable,
    read_cell_stream, read_stream_row,
};

pub fn load_god_texts(ctx: &mut AppContext) -> Result<(), Fault> {
    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"God1_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.god_intro_texts.clear();

        let mut row = 0i32;

        while row != 3 {
            read_stream_row(stm, b',');
            ctx.god_intro_texts.push([
                read_cell_stream(stm, 0).to_vec(),
                read_cell_stream(stm, 1).to_vec(),
            ]);
            row += 1;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"God2_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_stream_row(stm, b',');
        ctx.god_chatter_texts.clear();
        ctx.god_chatter_texts.push([
            read_cell_stream(stm, 0).to_vec(),
            read_cell_stream(stm, 1).to_vec(),
        ]);
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"God3_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_stream_row(stm, b',');
        ctx.god_short_texts = [
            read_cell_stream(stm, 0).to_vec(),
            read_cell_stream(stm, 1).to_vec(),
        ];
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"God4_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_stream_row(stm, b',');
        ctx.god_bought_texts = [
            read_cell_stream(stm, 0).to_vec(),
            read_cell_stream(stm, 1).to_vec(),
        ];
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"GodName_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_stream_row(stm, b',');
        ctx.god_name_text = read_cell_stream(stm, 0).to_vec();
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"GodItemName_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.god_item_names.clear();

        let mut row = 0i32;

        while row != 4 {
            read_stream_row(stm, b',');
            ctx.god_item_names.push(read_cell_stream(stm, 0).to_vec());
            row += 1;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"GodItemExplanation_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_stream_row(stm, b',');
        ctx.god_item_texts.clear();
        ctx.god_item_texts.push([
            read_cell_stream(stm, 0).to_vec(),
            read_cell_stream(stm, 1).to_vec(),
        ]);
    }

    Ok(())
}
