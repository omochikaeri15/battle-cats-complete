use super::{AssetStream, get_column_count, read_csv_cell};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GatyaDataSet {
    pub units: Vec<i32>,
    pub zeroed: [i32; 12],
    pub gap: i32,
    pub unset: [i32; 7],
    pub tail: i32,
    pub tail_flags: [u8; 2],
}

impl Default for GatyaDataSet {
    fn default() -> Self {
        Self {
            units: Vec::new(),
            zeroed: [0; 12],
            gap: 0,
            unset: [-1; 7],
            tail: 0,
            tail_flags: [0; 2],
        }
    }
}

pub fn parse_gatya_data_set_row(row: &mut GatyaDataSet, stm: &mut AssetStream<'_>) {
    row.units.clear();

    if get_column_count(stm) as i32 <= 0 {
        return;
    }

    let mut column = 0i32;

    while column < get_column_count(stm) as i32 {
        if read_csv_cell(stm, column) as i32 == -1 {
            return;
        }

        let value = read_csv_cell(stm, column) as i32;

        row.units.push(value);
        column += 1;
    }
}
