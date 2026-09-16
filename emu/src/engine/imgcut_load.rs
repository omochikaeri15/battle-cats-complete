use super::{read_asset_stream_line, read_csv_cell, read_csv_row, AssetStream, Cell};

pub const CUT_CELLS: usize = 4;

#[derive(Default)]
pub struct Imgcut {
    pub cuts: Vec<[i32; CUT_CELLS]>,
    pub png: String,
    pub cut: String,
    pub flag: i32,
}

pub fn imgcut_load(sheet: &mut Imgcut, png: &str, cut: &str, flag: i32, stm: Option<&mut AssetStream<'_>>) {
    sheet.png.clear();
    sheet.png.push_str(png);
    sheet.cut.clear();
    sheet.cut.push_str(cut);
    sheet.flag = flag;

    if sheet.cut.is_empty() {
        return;
    }

    let Some(stm) = stm else {
        return;
    };

    let mut discarded = Cell { at: 0, len: 0 };
    read_asset_stream_line(stm, &mut discarded);
    read_asset_stream_line(stm, &mut discarded);
    read_asset_stream_line(stm, &mut discarded);

    read_csv_row(stm);
    let cut_count = read_csv_cell(stm, 0) as i32;
    sheet.cuts.resize(cut_count as i64 as usize, [0; CUT_CELLS]);

    if cut_count <= 0 {
        return;
    }

    for index in 0..sheet.cuts.len() {
        read_csv_row(stm);

        sheet.cuts[index][0] = read_csv_cell(stm, 0) as i32;
        sheet.cuts[index][1] = read_csv_cell(stm, 1) as i32;
        sheet.cuts[index][2] = read_csv_cell(stm, 2) as i32;
        sheet.cuts[index][3] = read_csv_cell(stm, 3) as i32;
    }
}
