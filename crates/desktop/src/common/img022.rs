use iced::Task;

use kokoro::common::formats::SpriteSheet as CoreSpriteSheet;
use kokoro::Vfs;

use super::SpriteSheet;

pub fn ensure_loaded(sheets: &mut Vec<SpriteSheet>, vfs: &Vfs, pristine: bool) -> Task<(usize, Option<CoreSpriteSheet>)> {
    super::ensure_sheet_loaded(sheets, vfs, "img022", pristine)
}
