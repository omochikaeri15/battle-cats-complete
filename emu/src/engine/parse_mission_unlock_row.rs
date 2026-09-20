use super::{AssetStream, read_csv_cell};

pub fn parse_mission_unlock_row(row: &mut [i32; 10], stm: &AssetStream<'_>) {
    row[0] = read_csv_cell(stm, 0) as i32;
    row[1] = read_csv_cell(stm, 1) as i32;
    row[2] = read_csv_cell(stm, 2) as i32;
    row[3] = read_csv_cell(stm, 3) as i32;
    row[4] = read_csv_cell(stm, 4) as i32;
    row[5] = read_csv_cell(stm, 5) as i32;
    row[6] = read_csv_cell(stm, 6) as i32;
    row[7] = read_csv_cell(stm, 7) as i32;
    row[8] = read_csv_cell(stm, 8) as i32;
    row[9] = read_csv_cell(stm, 9) as i32;
}
