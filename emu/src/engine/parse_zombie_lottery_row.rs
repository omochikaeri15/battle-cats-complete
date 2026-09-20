use super::{AssetStream, get_column_count, read_csv_cell, read_csv_row};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ZombieLotteryRow {
    pub id: i32,
    pub interval: i32,
    pub entries: Vec<[i32; 4]>,
}

pub fn parse_zombie_lottery_row(row: &mut ZombieLotteryRow, stm: &mut AssetStream<'_>) {
    row.id = read_csv_cell(stm, 0) as i32;
    row.interval = (read_csv_cell(stm, 1) as i32).wrapping_mul(0x3c);
    row.entries.clear();

    while read_csv_row(stm) {
        if (get_column_count(stm) as i32) < 2 {
            return;
        }

        let cleared = read_csv_cell(stm, 0) as i32;
        let chance = read_csv_cell(stm, 2) as i32;
        let draws = read_csv_cell(stm, 1) as i32;
        let minimum = read_csv_cell(stm, 3) as i32;

        row.entries.push([cleared, chance, draws, minimum]);
    }
}
