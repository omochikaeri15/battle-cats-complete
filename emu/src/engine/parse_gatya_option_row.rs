use super::{AssetStream, GatyaDataSet, get_column_count, read_csv_cell};

pub fn parse_gatya_option_row(row: &mut GatyaDataSet, stm: &mut AssetStream<'_>) {
    row.banner_on = read_csv_cell(stm, 1) as i32 != 0;
    row.item_id_ticket = read_csv_cell(stm, 2) as i32;
    row.anime_id = read_csv_cell(stm, 3) as i32;
    row.btn_cut_id = read_csv_cell(stm, 4) as i32;
    row.series_id = read_csv_cell(stm, 5) as i32;
    row.menu_cut_id = read_csv_cell(stm, 6) as i32;

    let mut chara_id = -1i32;

    if get_column_count(stm) as i32 >= 8 {
        chara_id = read_csv_cell(stm, 7) as i32;
    }

    row.chara_id = chara_id;

    let mut wait_maanim_on = false;

    if get_column_count(stm) as i32 >= 9 {
        wait_maanim_on = read_csv_cell(stm, 8) as i32 != 0;
    }

    row.wait_maanim_on = wait_maanim_on;

    let mut img_id = -1i32;

    if get_column_count(stm) as i32 >= 10 {
        img_id = read_csv_cell(stm, 9) as i32;
    }

    row.img_id = img_id;
    row.tail = read_csv_cell(stm, 10) as i32;
}
