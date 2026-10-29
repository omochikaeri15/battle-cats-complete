use super::{AssetStream, get_column_count, read_csv_cell};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GatyaDataSet {
    pub units: Vec<i32>,
    pub units_2: Vec<i32>,
    pub units_3: Vec<i32>,
    pub series_id: i32,
    pub item_id_ticket: i32,
    pub anime_id: i32,
    pub btn_cut_id: i32,
    pub unset: i32,
    pub menu_cut_id: i32,
    pub chara_id: i32,
    pub img_id: i32,
    pub tail: i32,
    pub banner_on: bool,
    pub wait_maanim_on: bool,
}

impl Default for GatyaDataSet {
    fn default() -> Self {
        Self {
            units: Vec::new(),
            units_2: Vec::new(),
            units_3: Vec::new(),
            series_id: 0,
            item_id_ticket: -1,
            anime_id: -1,
            btn_cut_id: -1,
            unset: -1,
            menu_cut_id: -1,
            chara_id: -1,
            img_id: -1,
            tail: 0,
            banner_on: false,
            wait_maanim_on: false,
        }
    }
}

pub fn parse_gatya_data_set_row(row: &mut GatyaDataSet, stm: &mut AssetStream<'_>) {
    row.units.clear();

    if get_column_count(stm) as i32 <= 0 {
        return;
    }

    let mut column = 0i32;

    while column < get_column_count(stm) as i32 {
        if read_csv_cell(stm, column) as i32 == -1 {
            return;
        }

        let value = read_csv_cell(stm, column) as i32;

        row.units.push(value);
        column += 1;
    }
}
