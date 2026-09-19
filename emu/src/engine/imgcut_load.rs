use crate::Fault;

use super::{
    AppContext, AssetStream, Cell, load_sheet_png, open_asset_stream, query_localizable,
    read_asset_stream_line, read_csv_cell, read_csv_row, texture_clear,
};

pub const CUT_CELLS: usize = 4;

#[derive(Default)]
pub struct Imgcut {
    pub label: u64,
    pub cuts: Vec<[i32; CUT_CELLS]>,
    pub png: Vec<u8>,
    pub cut: Vec<u8>,
    pub flag: i32,
    pub width: i32,
    pub height: i32,
    pub whole: u8,
}

pub fn imgcut_load(
    ctx: &mut AppContext,
    sheet: &mut Imgcut,
    png: &[u8],
    cut: &[u8],
    flag: i32,
) -> Result<bool, Fault> {
    texture_clear(sheet);
    sheet.png = query_localizable(ctx, png);
    sheet.cut = query_localizable(ctx, cut);
    sheet.flag = flag;

    if !sheet.cut.is_empty() {
        let name = sheet.cut.clone();

        if let Some(bytes) = open_asset_stream(ctx, &name, 1, 1)? {
            let mut stream = AssetStream::new(&bytes, b'\n');
            let stm = &mut stream;

            let mut discarded = Cell { at: 0, len: 0 };
            read_asset_stream_line(stm, &mut discarded);
            read_asset_stream_line(stm, &mut discarded);
            read_asset_stream_line(stm, &mut discarded);

            read_csv_row(stm);
            let cut_count = read_csv_cell(stm, 0) as i32;
            sheet.cuts.resize(cut_count as i64 as usize, [0; CUT_CELLS]);

            if cut_count > 0 {
                for index in 0..cut_count as usize {
                    read_csv_row(stm);

                    sheet.cuts[index][0] = read_csv_cell(stm, 0) as i32;
                    sheet.cuts[index][1] = read_csv_cell(stm, 1) as i32;
                    sheet.cuts[index][2] = read_csv_cell(stm, 2) as i32;
                    sheet.cuts[index][3] = read_csv_cell(stm, 3) as i32;
                }
            }
        }
    }

    sheet.whole = 0;

    load_sheet_png(ctx, sheet)
}
