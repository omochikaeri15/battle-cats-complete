use crate::Fault;

use super::{CUT_CELLS, Imgcut};

pub fn imgcut_get_sprite_cut(sheet: &Imgcut, cut: i32) -> Result<&[i32; CUT_CELLS], Fault> {
    sheet
        .cuts
        .get(cut as i64 as usize)
        .ok_or(Fault::index_out_of_range(cut as i64, sheet.cuts.len() as i64))
}
