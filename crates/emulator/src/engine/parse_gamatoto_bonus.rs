use super::{AssetStream, get_column_count, read_csv_cell, read_csv_row};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct GamatotoBonus {
    pub header: [i32; 2],
    pub pairs: Vec<[i32; 2]>,
}

pub fn parse_gamatoto_bonus(out: &mut GamatotoBonus, stm: &mut AssetStream<'_>) {
    out.pairs.clear();
    read_csv_row(stm);
    out.header = [
        read_csv_cell(stm, 0) as i32,
        read_csv_cell(stm, 1) as i32,
    ];
    read_csv_row(stm);

    let mut column = 0i32;

    while column < get_column_count(stm) as i32 {
        let first = read_csv_cell(stm, column) as i32;
        let second = read_csv_cell(stm, column | 1) as i32;

        out.pairs.push([first, second]);
        column = column.wrapping_add(2);
    }
}
