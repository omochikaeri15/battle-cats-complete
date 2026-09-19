use crate::Fault;

use super::{
    AppContext, AssetStream, CastleRow, get_column_count, open_asset_stream, read_csv_cell,
    read_csv_row,
};

const TERMINATOR: i32 = -999;

pub fn load_enemy_castle_csv(ctx: &mut AppContext, which: i32) -> Result<(), Fault> {
    let name: Option<&[u8]> = match which {
        0 => Some(b"enemyCastleData0.csv"),
        1 => Some(b"enemyCastleData1.csv"),
        2 => Some(b"enemyCastleDataLegend.csv"),
        3 => Some(b"enemyCastleData2.csv"),
        _ => None,
    };
    let bytes = match name {
        Some(name) => open_asset_stream(ctx, name, 0, 0)?.unwrap_or_default(),
        None => Vec::new(),
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');
    let castle_vec = &mut ctx.enemy_castle;

    castle_vec.clear();

    while read_csv_row(stm) && read_csv_cell(stm, 0) as i32 != TERMINATOR {
        if get_column_count(stm) as i64 > 2 {
            let offset_x = read_csv_cell(stm, 0) as i32;
            let offset_y = read_csv_cell(stm, 1) as i32;
            let size = read_csv_cell(stm, 2) as i32;
            let art_variant = read_csv_cell(stm, 3) as i32;

            castle_vec.push(CastleRow {
                offset_x,
                offset_y,
                size,
                art_variant,
            });
            continue;
        }

        let offset_x = read_csv_cell(stm, 0) as i32;
        let offset_y = read_csv_cell(stm, 1) as i32;
        let size = read_csv_cell(stm, 2) as i32;

        castle_vec.push(CastleRow {
            offset_x,
            offset_y,
            size,
            art_variant: 0,
        });
    }

    Ok(())
}
