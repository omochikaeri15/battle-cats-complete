use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use kore::Source;

use iced::widget::image::Handle;

use kore::common::gfx::{autocrop, open_image};

#[derive(Clone)]
pub(crate) struct HeaderIcon {
    pub(crate) handle: Handle,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl HeaderIcon {
    pub(crate) fn dummy() -> Self {
        Self { handle: Handle::from_rgba(1, 1, vec![80, 80, 80, 255]), width: 1.0, height: 1.0 }
    }

    pub(crate) fn scale(&self, box_width: f32, box_height: f32) -> (f32, f32) {
        let scale = (box_width / self.width).min(box_height / self.height);

        (self.width * scale, self.height * scale)
    }
}

pub(crate) type Cache = RefCell<HashMap<PathBuf, Option<HeaderIcon>>>;

pub(crate) fn load(cache: &Cache, source: &Source) -> Option<HeaderIcon> {
    if let Some(cached) = cache.borrow().get(&source.path) {
        return cached.clone();
    }

    let icon = decode(source);
    cache.borrow_mut().insert(source.path.clone(), icon.clone());

    icon
}

fn decode(source: &Source) -> Option<HeaderIcon> {
    let img = open_image(source)?;
    let rgba = autocrop(img.to_rgba8());
    let (width, height) = rgba.dimensions();

    if width == 0 || height == 0 {
        return None;
    }

    Some(HeaderIcon {
        handle: Handle::from_rgba(width, height, rgba.into_raw()),
        width: width as f32,
        height: height as f32,
    })
}

