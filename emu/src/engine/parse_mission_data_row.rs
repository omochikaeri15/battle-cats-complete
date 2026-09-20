use std::rc::Rc;

use super::{
    AssetStream, MissionCondition, MissionConditionSetting, MissionItems, parse_mission_items,
    read_csv_cell,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MissionData {
    pub mission_id: i32,
    pub unlock_condition_id: i32,
    pub priority: i32,
    pub popup_condition: bool,
    pub condition_setting: Option<Rc<MissionConditionSetting>>,
    pub condition: MissionCondition,
    pub items: MissionItems,
}

pub fn parse_mission_data_row(out: &mut MissionData, stm: &AssetStream<'_>) {
    out.condition_setting = None;
    out.condition.values.clear();
    out.condition.gatya_setting = None;
    out.condition.limit_options.clear();
    out.items.items.clear();
    out.mission_id = read_csv_cell(stm, 0) as i32;
    out.unlock_condition_id = read_csv_cell(stm, 1) as i32;
    out.priority = read_csv_cell(stm, 2) as i32;
    out.popup_condition = read_csv_cell(stm, 3) as i32 == 1;

    let mut items = MissionItems::default();

    parse_mission_items(&mut items, stm, out.mission_id);

    out.items = items;
}
