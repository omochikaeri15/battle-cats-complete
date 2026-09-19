use super::Cell;

pub fn split_row_cells(bytes: &[u8], row: &Cell, delimiter: u8, cells: &mut Vec<Cell>) {
    cells.clear();

    if row.len == 0 {
        return;
    }

    let end = row.at + row.len;
    let mut cursor = row.at;

    loop {
        let hit = bytes[cursor..end]
            .iter()
            .position(|byte| *byte == delimiter)
            .map_or(end, |at| cursor + at);

        cells.push(Cell {
            at: cursor,
            len: hit - cursor,
        });

        if hit == end {
            return;
        }

        cursor = hit + 1;

        if cursor == end {
            return;
        }
    }
}
