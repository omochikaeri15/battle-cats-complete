use std::cell::RefCell;
use std::collections::HashMap;

use iced::widget::image::Handle;

use crate::common::SpriteSheet;

#[derive(Default)]
pub struct Cache {
    cache: RefCell<HashMap<usize, Handle>>,
}

impl Cache {
    pub(crate) fn clear(&self) {
        self.cache.borrow_mut().clear();
    }

    pub fn handle(&self, icon_id: usize, sheets: &[SpriteSheet]) -> Option<Handle> {
        if let Some(cached) = self.cache.borrow().get(&icon_id) {
            return Some(cached.clone());
        }

        for sheet in sheets {
            if !sheet.is_settled() {
                return None;
            }

            let Some(cropped) = sheet.core.crop(icon_id) else { continue; };
            let handle = Handle::from_rgba(cropped.width(), cropped.height(), cropped.into_raw());
            self.cache.borrow_mut().insert(icon_id, handle.clone());
            return Some(handle);
        }

        None
    }
}
