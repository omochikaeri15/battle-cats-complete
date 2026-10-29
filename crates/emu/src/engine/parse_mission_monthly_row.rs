use super::{AssetStream, get_column_count, read_csv_cell};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MissionMonthly {
    pub id: i32,
    pub mission_quantity: i32,
    pub group_number: i32,
    pub groups: Vec<i32>,
}

pub fn parse_mission_monthly_row(out: &mut MissionMonthly, stm: &AssetStream<'_>) {
    out.id = 0;
    out.mission_quantity = 0;
    out.group_number = 0;
    out.id = read_csv_cell(stm, 0) as i32;
    out.mission_quantity = read_csv_cell(stm, 1) as i32;
    out.group_number = read_csv_cell(stm, 2) as i32;
    out.groups.clear();

    let mut column = 3i32;

    while column < get_column_count(stm) as i32 {
        out.groups.push(read_csv_cell(stm, column) as i32);
        column += 1;
    }
}
