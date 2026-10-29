use super::{AssetStream, GatyaDataSet, get_column_count, read_csv_cell};

pub fn parse_gatya_data_set_row_3(row: &mut GatyaDataSet, stm: &mut AssetStream<'_>) {
    row.units_3.clear();

    if get_column_count(stm) as i32 <= 0 {
        return;
    }

    let mut column = 0i32;

    while column < get_column_count(stm) as i32 {
        if read_csv_cell(stm, column) as i32 == -1 {
            return;
        }

        let value = read_csv_cell(stm, column) as i32;

        row.units_3.push(value);
        column += 1;
    }
}
