use crate::Fault;

use super::{
    AppContext, AssetStream, format_localized, open_asset_stream, query_localizable,
    read_cell_stream, read_stream_row,
};

pub fn load_opening_ending_texts(ctx: &mut AppContext) -> Result<(), Fault> {
    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"ED_Message_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.ending_messages.clear();

        let mut row = 0i32;

        while row != 2 {
            read_stream_row(stm, b',');
            ctx.ending_messages.push([
                read_cell_stream(stm, 0).to_vec(),
                read_cell_stream(stm, 1).to_vec(),
                read_cell_stream(stm, 2).to_vec(),
                read_cell_stream(stm, 3).to_vec(),
            ]);
            row += 1;
        }
    }

    for (pattern, count, slot) in [
        (b"OP_%@.csv".as_slice(), 73i32, 0usize),
        (b"OPLegend_%@.csv".as_slice(), 73, 1),
        (b"OP2_%@.csv".as_slice(), 77, 2),
        (b"OP3_%@.csv".as_slice(), 77, 3),
    ] {
        let lang = query_localizable(ctx, b"lang");
        let name = format_localized(ctx, pattern, &lang)?;

        let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? else {
            continue;
        };
        let stm = &mut AssetStream::new(&bytes, b'\n');

        if let Some(lines) = ctx.opening_messages.get_mut(slot) {
            lines.clear();
        }

        let mut row = 0i32;

        while row != count {
            read_stream_row(stm, b',');

            let line = read_cell_stream(stm, 0).to_vec();

            if let Some(lines) = ctx.opening_messages.get_mut(slot) {
                lines.push(line);
            }

            row += 1;
        }
    }

    Ok(())
}
