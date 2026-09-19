use crate::Fault;

use super::{cell_is_int, get_column_count, open_asset_stream, read_asset_stream_line, read_csv_cell, read_csv_row, AppContext, AssetStream, Cell};

pub fn setup_bg_color(ctx: &mut AppContext, background: i32) -> Result<(), Fault> {
    let Some(bytes) = open_asset_stream(ctx, b"bg.csv", 0, 0)? else {
        return Ok(());
    };
    let mut stm = AssetStream::new(&bytes, b'\n');
    let mut line = Cell { at: 0, len: 0 };

    read_asset_stream_line(&mut stm, &mut line);

    while read_csv_row(&mut stm) {
        if read_csv_cell(&stm, 0) as i32 != background {
            continue;
        }

        for color in 0..4usize {
            let red = read_csv_cell(&stm, color as i32 * 3 + 1) as i32;
            let green = read_csv_cell(&stm, color as i32 * 3 + 2) as i32;
            let blue = read_csv_cell(&stm, color as i32 * 3 + 3) as i32;

            ctx.set_i32_at(AppContext::BG_SETUP + color * 4, (green << 8) | (red << 0x10) | blue)?;
        }

        ctx.set_i32_at(AppContext::BG_SETUP + 0x10, read_csv_cell(&stm, 0xd) as i32)?;
        ctx.set_block_at::<1>(AppContext::BG_SETUP + 0x14, [(read_csv_cell(&stm, 0xe) as i32 != 0) as u8])?;

        let effect = if get_column_count(&stm) as i64 >= 0x10 && cell_is_int(&stm, 0xf) { read_csv_cell(&stm, 0xf) as i32 } else { -1 };

        ctx.set_i32_at(AppContext::BG_SETUP + 0x18, effect)?;
        ctx.set_block_at::<8>(AppContext::BG_SETUP + 0x1c, [0; 8])?;

        if get_column_count(&stm) as i64 >= 0x14 && cell_is_int(&stm, 0x13) {
            let red = read_csv_cell(&stm, 0x10) as i32;
            let green = read_csv_cell(&stm, 0x11) as i32;
            let blue = read_csv_cell(&stm, 0x12) as i32;
            let alpha = read_csv_cell(&stm, 0x13) as i32;
            let top = blue | (alpha << 0x18) | (green << 8) | (red << 0x10);

            ctx.set_i32_at(AppContext::BG_SETUP + 0x1c, top)?;

            let bottom = if get_column_count(&stm) as i64 >= 0x18 && cell_is_int(&stm, 0x17) {
                let red = read_csv_cell(&stm, 0x14) as i32;
                let green = read_csv_cell(&stm, 0x15) as i32;
                let blue = read_csv_cell(&stm, 0x16) as i32;
                let alpha = read_csv_cell(&stm, 0x17) as i32;

                blue | (alpha << 0x18) | (green << 8) | (red << 0x10)
            } else {
                ctx.i32_at(AppContext::BG_SETUP + 0x1c)?
            };

            ctx.set_i32_at(AppContext::BG_SETUP + 0x20, bottom)?;
        }

        break;
    }

    Ok(())
}
