use super::{AssetStream, get_column_count, read_csv_cell};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct DojoChestRow {
    pub gatya_id: i32,
    pub cost: i32,
    pub drop_items: Vec<i32>,
}

pub fn parse_dojo_chest_row(stm: &mut AssetStream<'_>) -> DojoChestRow {
    let mut row = DojoChestRow {
        gatya_id: read_csv_cell(stm, 0) as i32,
        cost: read_csv_cell(stm, 1) as i32,
        drop_items: Vec::new(),
    };
    let mut column = 2i32;

    while column < get_column_count(stm) as i32 {
        let value = read_csv_cell(stm, column) as i32;

        if value.wrapping_sub(0x7530) as u32 <= 0x270f {
            row.drop_items.push(value);
        }

        column += 1;
    }

    row
}
