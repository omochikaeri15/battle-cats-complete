use std::cell::RefCell;
use std::collections::HashMap;

use iced::widget::image::Handle;
use codex::cat::unit::TalentGroup;

use kokoro::common::gfx::{autocrop, open_image};
use kokoro::Vfs;

pub(crate) type Cache = RefCell<HashMap<String, Option<Handle>>>;

pub(crate) fn load(cache: &Cache, group: &TalentGroup, vfs: &Vfs, pristine: bool) -> Option<Handle> {
    let image_id = if group.name_id > 0 { group.name_id } else { i16::from(group.ability_id) };

    if image_id <= 0 {
        return None;
    }

    let name = format!("Skill_name_{:03}.png", image_id);
    let path = if pristine { vfs.pristine(&name) } else { vfs.find(&name) }?;
    let key = path.file_name()?.to_string_lossy().into_owned();

    if let Some(cached) = cache.borrow().get(&key) {
        return cached.clone();
    }

    let handle = open_image(&vfs.source(&path)).map(|image| {
        let rgba = autocrop(image.to_rgba8());

        Handle::from_rgba(rgba.width(), rgba.height(), rgba.into_raw())
    });

    cache.borrow_mut().insert(key, handle.clone());

    handle
}
