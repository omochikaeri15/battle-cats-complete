use super::{AssetStream, STAGE_ENEMY_COLUMNS, get_column_count, read_csv_cell};

pub fn parse_stage_enemy_row(entry: &mut [i32; STAGE_ENEMY_COLUMNS], stm: &AssetStream<'_>) -> i64 {
    *entry = [0; STAGE_ENEMY_COLUMNS];

    let mut result = 0;
    let mut column = 0;

    while column != STAGE_ENEMY_COLUMNS as i32 {
        result = get_column_count(stm) as i64;

        if column as i64 >= result {
            break;
        }

        result = read_csv_cell(stm, column);
        entry[column as usize] = result as i32;
        column += 1;
    }

    entry[2] = entry[2].wrapping_add(entry[2]);
    entry[3] = entry[3].wrapping_add(entry[3]);
    entry[4] <<= 1;

    result
}
