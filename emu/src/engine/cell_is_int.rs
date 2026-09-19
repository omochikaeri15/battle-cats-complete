use crate::operation;

use super::{get_cell, get_column_count, read_cell_stream, AssetStream};

pub fn cell_is_int(stm: &AssetStream<'_>, idx: i32) -> bool {
    if get_column_count(stm) as i32 <= idx {
        return false;
    }

    let text = read_cell_stream(stm, idx);
    let value = get_cell(stm, idx).map_or(0, |cell| operation::strtol(&stm.bytes[cell.at..], 0xa).value as i32);

    value.to_string().as_bytes() == text
}
