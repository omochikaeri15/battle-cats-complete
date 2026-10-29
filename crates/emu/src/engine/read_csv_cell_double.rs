use crate::ops;

use super::{AssetStream, get_cell};

pub fn read_csv_cell_double(stm: &AssetStream<'_>, col: i32) -> f64 {
    let Some(cell) = get_cell(stm, col) else {
        return 0.0;
    };

    ops::strtod(&stm.bytes[cell.at..]).map_or(0.0, |(value, _)| value)
}
