use super::Surface;

pub fn texture_get_height(surface: Surface<'_>) -> i32 {
    match surface {
        Surface::Sheet(sheet) => sheet.height,
        Surface::Label(label) => label.height,
    }
}
