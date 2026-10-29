use super::{AssetStream, Cell, read_asset_stream_line, split_row_cells};

pub fn read_stream_row(stm: &mut AssetStream<'_>, delimiter: u8) -> bool {
    let mut line = Cell { at: 0, len: 0 };

    if !read_asset_stream_line(stm, &mut line) {
        return false;
    }

    let bytes = stm.bytes;
    split_row_cells(bytes, &line, delimiter, &mut stm.cells);

    true
}
