use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use emu::engine::{AssetSource, SheetImage};
use kore::Vfs;
use rayon::prelude::*;
use tracing::{debug, info, warn};

#[derive(Clone)]
pub struct Sheet {
    pub width: u32,
    pub height: u32,
    pub pixels: Arc<[u8]>,
}

pub type SheetCache = BTreeMap<Box<str>, Sheet>;

pub type FileIndex = BTreeMap<Box<str>, PathBuf>;

pub struct DiskAssets {
    files: Rc<RefCell<FileIndex>>,
    sheets: Rc<RefCell<SheetCache>>,
    missing: Vec<Box<str>>,
}

impl DiskAssets {
    pub fn new(files: Rc<RefCell<FileIndex>>, sheets: Rc<RefCell<SheetCache>>) -> Self {
        Self {
            files,
            sheets,
            missing: Vec::new(),
        }
    }

    pub fn index(vfs: &Vfs) -> FileIndex {
        let started = std::time::Instant::now();
        let listed = vfs.glob("");
        let mut files: FileIndex = listed
            .par_iter()
            .filter_map(|name| vfs.locate(name).map(|path| (name.clone(), path)))
            .collect();

        let bare: Vec<Box<str>> = files
            .keys()
            .filter_map(|name| {
                let (stem, extension) = name.rsplit_once('.')?;
                let (head, code) = stem.rsplit_once('_')?;

                (code.len() == 2 && code.chars().all(|letter| letter.is_ascii_lowercase()))
                    .then(|| Box::from(format!("{head}.{extension}").as_str()))
            })
            .filter(|name| !files.contains_key(name))
            .collect();

        let localized = bare
            .into_par_iter()
            .filter_map(|name| {
                let best = vfs.variants(&name).into_iter().next()?;

                vfs.locate(&best).map(|path| (name, path))
            })
            .collect::<Vec<_>>();

        let aliased = localized.len();

        files.extend(localized);

        info!(
            "emu: {} files resolved in {:?}, {} through a regional variant",
            files.len(),
            started.elapsed(),
            aliased
        );

        files
    }

    fn read(&self, name: &str) -> Option<Vec<u8>> {
        let path = self.files.borrow().get(name).cloned()?;

        std::fs::read(path).ok()
    }

    fn decode(&mut self, name: &str) -> Option<Sheet> {
        if let Some(sheet) = self.sheets.borrow().get(name) {
            return Some(sheet.clone());
        }

        let bytes = self.read(name)?;
        let decoded = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
            .inspect_err(|error| warn!("emu: {name} failed to decode: {error}"))
            .ok()?
            .to_rgba8();
        let sheet = Sheet {
            width: decoded.width(),
            height: decoded.height(),
            pixels: Arc::from(decoded.into_raw().as_slice()),
        };

        self.sheets
            .borrow_mut()
            .insert(Box::from(name), sheet.clone());

        Some(sheet)
    }
}

impl AssetSource for DiskAssets {
    fn open(&mut self, name: &[u8], _packed: u8, _encrypted: u8) -> Option<Vec<u8>> {
        let name = std::str::from_utf8(name).ok()?;
        let found = self.read(name);

        if found.is_none() && !self.missing.iter().any(|seen| seen.as_ref() == name) {
            debug!("emu: no file named {name}");
            self.missing.push(Box::from(name));
        }

        found
    }

    fn load_png(&mut self, name: &[u8]) -> Option<SheetImage> {
        let name = std::str::from_utf8(name).ok()?;
        let sheet = self.decode(name)?;

        Some(SheetImage {
            width: sheet.width as i32,
            height: sheet.height as i32,
            whole: 0,
        })
    }

    fn upload(&mut self, name: &[u8]) {
        if let Ok(name) = std::str::from_utf8(name) {
            self.decode(name);
        }
    }

    fn pack_entry(&mut self, _name: &[u8]) -> Vec<u8> {
        Vec::new()
    }
}
