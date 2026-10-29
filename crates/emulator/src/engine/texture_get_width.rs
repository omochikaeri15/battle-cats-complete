use super::Surface;

pub fn texture_get_width(surface: Surface<'_>) -> i32 {
    match surface {
        Surface::Sheet(sheet) => sheet.width,
        Surface::Label(label) => label.width,
    }
}
