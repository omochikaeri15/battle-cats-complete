use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::path::Path;
use std::rc::Rc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use dioxus::core::spawn_forever;
use dioxus::prelude::*;
use image::{ImageFormat, RgbaImage};
use tracing::{debug, warn};

use kokoro::common::formats::{self, SpriteSheet};
use kokoro::common::gfx;
use kokoro::Vfs;

use crate::platform::{Picture, Platform};

const PACE_MS: u32 = 40;
const PER_PACE: usize = 6;

struct Wanted {
    key: String,
    kind: Picture,
    name: Box<str>,
    folder: String,
}

#[derive(Clone)]
pub(crate) struct Pictures {
    platform: Platform,
    urls: Rc<RefCell<HashMap<String, Rc<str>>>>,
    sizes: Rc<RefCell<HashMap<String, (u32, u32)>>>,
    sheets: Rc<RefCell<HashMap<String, Rc<Vec<SpriteSheet>>>>>,
    banner_frame: Rc<RefCell<Option<Rc<RgbaImage>>>>,
    wanted: Rc<RefCell<Vec<Wanted>>>,
    inflight: Rc<RefCell<HashSet<String>>>,
    failed: Rc<RefCell<HashSet<String>>>,
    scheduling: Rc<Cell<bool>>,
    bumping: Rc<Cell<bool>>,
    tick: Signal<u32>,
}

pub(crate) struct Sized {
    pub(crate) url: Rc<str>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

fn file_name(path: &Path) -> Option<Box<str>> {
    path.file_name().and_then(|name| name.to_str()).map(Box::from)
}

fn encode(image: &RgbaImage) -> Option<Vec<u8>> {
    let mut png = Vec::new();

    image.write_to(&mut Cursor::new(&mut png), ImageFormat::Png).inspect_err(|err| warn!("An image could not be encoded: {}", err)).ok()?;

    Some(png)
}

fn data_url(png: &[u8]) -> Rc<str> {
    Rc::from(format!("data:image/png;base64,{}", STANDARD.encode(png)))
}

pub(crate) fn provide(platform: &Platform) -> Pictures {
    let platform = platform.clone();

    use_context_provider(move || Pictures {
        platform,
        urls: Rc::default(),
        sizes: Rc::default(),
        sheets: Rc::default(),
        banner_frame: Rc::default(),
        wanted: Rc::default(),
        inflight: Rc::default(),
        failed: Rc::default(),
        scheduling: Rc::default(),
        bumping: Rc::default(),
        tick: Signal::new(0),
    })
}

pub(crate) fn pictures() -> Pictures {
    use_context::<Pictures>()
}

impl Pictures {
    pub(crate) fn forget_all(&self) {
        self.urls.borrow_mut().clear();
        self.sizes.borrow_mut().clear();
        self.sheets.borrow_mut().clear();
        self.wanted.borrow_mut().clear();
        self.failed.borrow_mut().clear();
        self.bump();
    }

    pub(crate) fn watch(&self) -> u32 {
        (self.tick)()
    }

    pub(crate) fn settled(&self) -> bool {
        self.wanted.borrow().is_empty() && self.inflight.borrow().is_empty()
    }

    fn bump(&self) {
        if self.bumping.replace(true) {
            return;
        }

        let platform = self.platform.clone();
        let bumping = Rc::clone(&self.bumping);
        let mut tick = self.tick;

        spawn_forever(async move {
            platform.host().frame().await;
            bumping.set(false);
            tick += 1;
        });
    }

    fn remote(&self, key: String, kind: Picture, name: Box<str>, folder: String) -> Option<Rc<str>> {
        let _ = (self.tick)();

        if let Some(url) = self.urls.borrow().get(&key) {
            return Some(Rc::clone(url));
        }

        if self.failed.borrow().contains(&key) || self.inflight.borrow().contains(&key) || self.wanted.borrow().iter().any(|held| held.key == key) {
            return None;
        }

        self.wanted.borrow_mut().push(Wanted { key, kind, name, folder });
        self.schedule();

        None
    }

    pub(crate) fn unwant(&self, key: &str) {
        self.wanted.borrow_mut().retain(|held| held.key != key);
    }

    fn schedule(&self) {
        if self.scheduling.replace(true) {
            return;
        }

        let pictures = self.clone();

        spawn_forever(async move {
            loop {
                pictures.platform.host().sleep(PACE_MS).await;

                let batch: Vec<Wanted> = {
                    let mut wanted = pictures.wanted.borrow_mut();
                    let take = wanted.len().min(PER_PACE);

                    wanted.drain(..take).collect()
                };

                for held in batch {
                    if pictures.urls.borrow().contains_key(&held.key) || !pictures.inflight.borrow_mut().insert(held.key.clone()) {
                        continue;
                    }

                    pictures.request(held);
                }
            }
        });
    }

    fn request(&self, held: Wanted) {
        let pending = self.platform.host().picture(held.kind, held.name.clone());
        let pictures = self.clone();
        let started = self.platform.host().now();

        spawn_forever(async move {
            match pending.await {
                Ok(Some(prepared)) => {
                    let url = if pictures.platform.host().can_publish() {
                        match pictures.platform.host().publish_image(&held.folder, &held.name, prepared.png.clone()).await {
                            Ok(Some(url)) => Rc::from(url),
                            Ok(None) => data_url(&prepared.png),
                            Err(fault) => {
                                warn!("An image could not be published: {fault}");
                                data_url(&prepared.png)
                            }
                        }
                    } else {
                        data_url(&prepared.png)
                    };

                    pictures.sizes.borrow_mut().insert(held.key.clone(), (prepared.width, prepared.height));
                    pictures.urls.borrow_mut().insert(held.key.clone(), url);
                    debug!(key = %held.key, ms = (pictures.platform.host().now() - started) as u32, "A picture is ready");
                }
                Ok(None) => {
                    warn!("The host has no image for {}", held.key);
                    pictures.failed.borrow_mut().insert(held.key.clone());
                }
                Err(fault) => {
                    warn!("An image could not be prepared: {fault}");
                    pictures.failed.borrow_mut().insert(held.key.clone());
                }
            }

            pictures.inflight.borrow_mut().remove(&held.key);
            pictures.bump();
        });
    }

    pub(crate) fn trimmed(&self, path: &Path) -> Option<Sized> {
        let name = file_name(path)?;
        let key = format!("trim:{name}");
        let url = self.remote(key.clone(), Picture::Trimmed, name, "trimmed".to_owned())?;
        let (width, height) = *self.sizes.borrow().get(&key)?;

        Some(Sized { url, width, height })
    }

    pub(crate) fn boxed(&self, path: &Path, canvas: u32) -> Option<Rc<str>> {
        let name = file_name(path)?;

        self.remote(format!("box{canvas}:{name}"), Picture::Boxed(canvas), name, format!("boxed{canvas}"))
    }

    pub(crate) fn cut(&self, sheet: &str, icon: usize) -> Option<Rc<str>> {
        let name: Box<str> = format!("{sheet}_{icon}.png").into();

        self.remote(format!("cut:{sheet}:{icon}"), Picture::Cut { sheet: Box::from(sheet), icon: icon as u32 }, name, "cut".to_owned())
    }

    pub(crate) fn cut_sized(&self, sheet: &str, icon: usize) -> Option<Sized> {
        let url = self.cut(sheet, icon)?;
        let (width, height) = *self.sizes.borrow().get(&format!("cut:{sheet}:{icon}"))?;

        Some(Sized { url, width, height })
    }

    pub(crate) fn banner_key(kind: &Picture, path: &Path) -> Option<String> {
        Some(format!("{kind:?}:{}", file_name(path)?))
    }

    pub(crate) fn banner(&self, kind: Picture, path: &Path) -> Option<Rc<str>> {
        let name = file_name(path)?;
        let folder = match kind {
            Picture::EnemyBanner => "enemy",
            _ => "banner",
        };

        self.remote(format!("{kind:?}:{name}"), kind, name, folder.to_owned())
    }

    fn local(&self, key: String, build: impl FnOnce() -> Option<RgbaImage>) -> Option<Rc<str>> {
        if let Some(url) = self.urls.borrow().get(&key) {
            return Some(Rc::clone(url));
        }

        let url = data_url(&encode(&build()?)?);

        self.urls.borrow_mut().insert(key, Rc::clone(&url));

        Some(url)
    }

    pub(crate) fn frame_url(&self) -> Option<Rc<str>> {
        self.local("banner:frame".to_owned(), || self.frame().map(|frame| (*frame).clone()))
    }

    fn frame(&self) -> Option<Rc<RgbaImage>> {
        if let Some(frame) = self.banner_frame.borrow().as_ref() {
            return Some(Rc::clone(frame));
        }

        let frame = Rc::new(gfx::banner_frame()?);

        *self.banner_frame.borrow_mut() = Some(Rc::clone(&frame));

        Some(frame)
    }

    pub(crate) fn asset(&self, name: &str, bytes: &[u8]) -> Option<Rc<str>> {
        self.local(format!("asset:{name}"), || Some(image::load_from_memory(bytes).ok()?.to_rgba8()))
    }

    pub(crate) fn sheets(&self, vfs: &Vfs, name: &str) -> Rc<Vec<SpriteSheet>> {
        if let Some(sheets) = self.sheets.borrow().get(name) {
            return Rc::clone(sheets);
        }

        let layers = Rc::new(formats::sheets(vfs, name));
        let settled = !layers.is_empty() && layers.iter().all(|layer| layer.image_data.is_some());

        if settled {
            self.sheets.borrow_mut().insert(name.to_owned(), Rc::clone(&layers));
        }

        layers
    }
}
