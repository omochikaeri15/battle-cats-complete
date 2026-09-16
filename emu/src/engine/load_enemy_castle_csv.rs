use super::{get_column_count, read_csv_cell, read_csv_row, AssetStream, CastleRow};

const TERMINATOR: i32 = -999;

pub fn load_enemy_castle_csv(castle_vec: &mut Vec<CastleRow>, stm: &mut AssetStream<'_>) {
    castle_vec.clear();

    while read_csv_row(stm) && read_csv_cell(stm, 0) as i32 != TERMINATOR {
        if get_column_count(stm) as i64 > 2 {
            let offset_x = read_csv_cell(stm, 0) as i32;
            let offset_y = read_csv_cell(stm, 1) as i32;
            let size = read_csv_cell(stm, 2) as i32;
            let art_variant = read_csv_cell(stm, 3) as i32;

            castle_vec.push(CastleRow { offset_x, offset_y, size, art_variant });
            continue;
        }

        let offset_x = read_csv_cell(stm, 0) as i32;
        let offset_y = read_csv_cell(stm, 1) as i32;
        let size = read_csv_cell(stm, 2) as i32;

        castle_vec.push(CastleRow { offset_x, offset_y, size, art_variant: 0 });
    }
}
