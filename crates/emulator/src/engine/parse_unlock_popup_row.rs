use super::{AssetStream, read_csv_cell};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct UnlockPopupRow {
    pub popup_id: i32,
    pub enabled: u8,
    pub stage: i32,
    pub map_conditions: i32,
    pub user_rank: i32,
    pub get_char_id_1: i32,
    pub get_char_id_2: i32,
    pub item_id: i32,
    pub item_count: i32,
    pub skill: i32,
    pub os_id: i32,
    pub unlock_eye_1_id: i32,
    pub unlock_eye_2_id: i32,
    pub add_lv_1: i32,
    pub add_lv_2: i32,
    pub unlock_plus_id: i32,
    pub add_lv: i32,
    pub conditions: i32,
    pub help_enabled: u8,
}

pub fn parse_unlock_popup_row(row: &mut UnlockPopupRow, stm: &AssetStream<'_>) {
    row.popup_id = read_csv_cell(stm, 0) as i32;
    row.enabled = u8::from(read_csv_cell(stm, 1) == 1);
    row.stage = read_csv_cell(stm, 3) as i32;
    row.map_conditions = read_csv_cell(stm, 4) as i32;
    row.user_rank = read_csv_cell(stm, 5) as i32;
    row.get_char_id_1 = read_csv_cell(stm, 6) as i32;
    row.get_char_id_2 = read_csv_cell(stm, 7) as i32;
    row.os_id = read_csv_cell(stm, 8) as i32;
    row.unlock_eye_1_id = read_csv_cell(stm, 9) as i32;
    row.add_lv_1 = read_csv_cell(stm, 0xa) as i32;
    row.unlock_eye_2_id = read_csv_cell(stm, 0xb) as i32;
    row.add_lv_2 = read_csv_cell(stm, 0xc) as i32;
    row.unlock_plus_id = read_csv_cell(stm, 0xd) as i32;
    row.add_lv = read_csv_cell(stm, 0xe) as i32;
    row.skill = read_csv_cell(stm, 0xf) as i32;
    row.item_id = read_csv_cell(stm, 0x10) as i32;
    row.item_count = read_csv_cell(stm, 0x11) as i32;
    row.conditions = read_csv_cell(stm, 2) as i32;
    row.help_enabled = u8::from(read_csv_cell(stm, 0x12) != 0);
}
