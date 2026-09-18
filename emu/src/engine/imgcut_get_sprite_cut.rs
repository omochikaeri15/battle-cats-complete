use crate::Fault;

use super::{Imgcut, CUT_CELLS};

pub fn imgcut_get_sprite_cut(sheet: &Imgcut, cut: i32) -> Result<&[i32; CUT_CELLS], Fault> {
    sheet.cuts.get(cut as i64 as usize).ok_or(Fault::IndexOutOfRange {
        site: "imgcut_get_sprite_cut",
        index: cut as i64,
        limit: sheet.cuts.len() as i64,
    })
}
