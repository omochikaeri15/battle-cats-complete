use std::rc::Rc;

use super::{
    AssetStream, MissionGatyaSetting, MissionLimitOption, get_column_count, read_cell_stream,
    read_csv_cell,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MissionCondition {
    pub mission_type: i32,
    pub conditions_type: i32,
    pub values: Vec<i32>,
    pub progress_count: i32,
    pub gatya_setting: Option<Rc<MissionGatyaSetting>>,
    pub limit_options: Vec<Rc<MissionLimitOption>>,
}

pub fn parse_mission_condition_row(out: &mut MissionCondition, stm: &AssetStream<'_>) {
    out.values.clear();
    out.gatya_setting = None;
    out.limit_options.clear();
    out.mission_type = read_csv_cell(stm, 1) as i32;
    out.conditions_type = read_csv_cell(stm, 2) as i32;
    out.progress_count = read_csv_cell(stm, 4) as i32;
    out.values.clear();

    let mut column = 5i32;

    while column < get_column_count(stm) as i32 {
        if read_cell_stream(stm, column).is_empty() {
            column += 1;

            continue;
        }

        let value = read_csv_cell(stm, column) as i32;

        out.values.push(value);
        column += 1;
    }
}
