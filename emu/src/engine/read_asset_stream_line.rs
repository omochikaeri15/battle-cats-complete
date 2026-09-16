use super::{AssetStream, Cell};

pub fn read_asset_stream_line(stm: &mut AssetStream<'_>, out: &mut Cell) -> bool {
    if stm.cursor >= stm.end {
        stm.cells.clear();

        return false;
    }

    let hit = stm.bytes[stm.cursor..stm.end]
        .iter()
        .position(|byte| *byte == stm.line_delimiter)
        .map_or(stm.end, |at| stm.cursor + at);

    let len = hit - stm.cursor;

    out.at = stm.cursor;
    out.len = len;
    stm.cursor = hit + 1;

    let mut i = len + 1;

    out.len = loop {
        if i == 1 {
            break len;
        }

        let byte = stm.bytes[out.at + i - 2];
        i -= 1;

        if byte == b'\n' || byte == b'\r' {
            continue;
        }

        if i == 0 {
            break len;
        }

        break len.min(i);
    };

    true
}
