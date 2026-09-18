use crate::operation;

use super::{get_cell, AssetStream};

pub fn read_csv_cell(stm: &AssetStream<'_>, col: i32) -> i64 {
    let Some(cell) = get_cell(stm, col) else {
        return 0;
    };

    operation::strtol(&stm.bytes[cell.at..], 0xa).value
}
