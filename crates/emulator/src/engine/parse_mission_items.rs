use super::{AssetStream, get_column_count, read_csv_cell};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MissionItems {
    pub id: i32,
    pub items: Vec<[i32; 3]>,
}

pub fn parse_mission_items(out: &mut MissionItems, stm: &AssetStream<'_>, id: i32) {
    out.items.clear();
    out.id = id;

    let mut column = 4i32;

    while column < get_column_count(stm) as i32 {
        let category = read_csv_cell(stm, column) as i32;
        let item = read_csv_cell(stm, column.wrapping_add(1)) as i32;
        let count = read_csv_cell(stm, column.wrapping_add(2)) as i32;

        out.items.push([category, item, count]);
        column = column.wrapping_add(3);
    }
}
