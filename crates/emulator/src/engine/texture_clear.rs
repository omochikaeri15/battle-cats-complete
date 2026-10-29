use super::Imgcut;

pub fn texture_clear(sheet: &mut Imgcut) {
    sheet.cuts.clear();
    sheet.png.clear();
    sheet.cut.clear();
}
