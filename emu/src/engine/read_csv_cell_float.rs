use crate::operation;

use super::{AssetStream, get_cell};

pub fn read_csv_cell_float(stm: &AssetStream<'_>, col: i32) -> f32 {
    let Some(cell) = get_cell(stm, col) else {
        return 0.0;
    };

    operation::strtof(&stm.bytes[cell.at..]).map_or(0.0, |(value, _)| value)
}
