use super::{AssetStream, get_column_count, read_csv_cell, read_csv_row};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct RealmsRngTable {
    pub rows: Vec<[i32; 6]>,
    pub altar_id: i32,
}

pub fn parse_realms_rng_table(stm: &mut AssetStream<'_>) -> RealmsRngTable {
    let mut table = RealmsRngTable::default();

    read_csv_row(stm);

    table.altar_id = read_csv_cell(stm, 0) as i32;

    while read_csv_row(stm) {
        if (get_column_count(stm) as i32) < 6 {
            break;
        }

        let first = read_csv_cell(stm, 0) as i32;
        let second = read_csv_cell(stm, 1) as i32;
        let third = read_csv_cell(stm, 2) as i32;
        let fourth = read_csv_cell(stm, 3) as i32;
        let fifth = read_csv_cell(stm, 4) as i32;
        let sixth = read_csv_cell(stm, 5) as i32;

        table
            .rows
            .push([first, second, third, fourth, fifth, sixth]);
    }

    table
}
