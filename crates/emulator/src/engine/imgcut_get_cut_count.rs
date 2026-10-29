use super::Imgcut;

pub fn imgcut_get_cut_count(sheet: &Imgcut) -> u64 {
    sheet.cuts.len() as u64
}
