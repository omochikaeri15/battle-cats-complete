use super::{AssetStream, get_cell};

pub fn read_cell_stream<'s>(stm: &'s AssetStream<'_>, idx: i32) -> &'s [u8] {
    let Some(cell) = get_cell(stm, idx) else {
        return &[];
    };

    &stm.bytes[cell.at..cell.at + cell.len]
}
