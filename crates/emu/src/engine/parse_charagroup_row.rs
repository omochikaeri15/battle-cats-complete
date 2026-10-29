use crate::Fault;

use super::{AssetStream, CharaGroup, get_column_count, read_cell_stream, read_csv_cell};

pub fn parse_charagroup_row(group: &mut CharaGroup, stm: &AssetStream<'_>) -> Result<(), Fault> {
    group.units.clear();
    group.name_key.clear();
    group.kind = 0;
    group.name_key = read_cell_stream(stm, 1).to_vec();
    group.kind = read_csv_cell(stm, 2) as i32;

    let mut column = 3i32;

    while column < get_column_count(stm) as i32 {
        let unit_id = read_csv_cell(stm, column) as i32;

        group.units.push(unit_id);
        column += 1;
    }

    Ok(())
}
