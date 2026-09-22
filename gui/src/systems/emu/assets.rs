use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::mem;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use emu::engine::{AssetSource, SheetImage};
use kore::common::io::APP_LANGUAGES;
use kore::Vfs;
use nyanko::combat::Separator;
use rayon::prelude::*;
use tracing::{debug, info, warn};

#[derive(Clone)]
pub struct Sheet {
    pub width: u32,
    pub height: u32,
    pub pixels: Arc<[u8]>,
    pub stamp: u64,
}

static NEXT_STAMP: AtomicU64 = AtomicU64::new(1);

impl Sheet {
    pub fn new(width: u32, height: u32, pixels: &[u8]) -> Self {
        Self { width, height, pixels: Arc::from(pixels), stamp: NEXT_STAMP.fetch_add(1, Ordering::Relaxed) }
    }
}

pub(super) const WIDE_COMMA: &str = "\u{ff0c}";

pub type SheetCache = BTreeMap<Box<str>, Sheet>;

pub type FileIndex = BTreeMap<Box<str>, PathBuf>;

fn stripped(name: &str) -> Option<(&str, &str)> {
    let (stem, extension) = name.rsplit_once('.')?;
    let (head, code) = stem.rsplit_once('_')?;

    APP_LANGUAGES.iter().any(|&(language, _)| language == code).then_some((head, extension))
}

#[derive(Default)]
pub struct Ledger {
    armed: bool,
    seen: BTreeSet<Box<str>>,
    fresh: Vec<(Box<str>, PathBuf)>,
}

impl Ledger {
    pub fn arm(&mut self) {
        self.armed = true;
        self.seen.clear();
        self.fresh.clear();
    }

    pub fn disarm(&mut self) {
        self.armed = false;
        self.seen.clear();
        self.fresh.clear();
    }

    pub fn note(&mut self, name: &str, path: &Path) {
        if self.armed && !self.seen.contains(name) {
            self.seen.insert(Box::from(name));
            self.fresh.push((Box::from(name), path.to_path_buf()));
        }
    }

    fn wants(&self, name: &str) -> bool {
        self.armed && !self.seen.contains(name)
    }

    pub fn drain(&mut self) -> Vec<(Box<str>, PathBuf)> {
        mem::take(&mut self.fresh)
    }
}

pub type SharedLedger = Rc<RefCell<Ledger>>;

pub struct DiskAssets {
    files: Rc<RefCell<FileIndex>>,
    sheets: Rc<RefCell<SheetCache>>,
    ledger: SharedLedger,
    missing: Vec<Box<str>>,
}

impl DiskAssets {
    pub fn new(files: Rc<RefCell<FileIndex>>, sheets: Rc<RefCell<SheetCache>>, ledger: SharedLedger) -> Self {
        Self {
            files,
            sheets,
            ledger,
            missing: Vec::new(),
        }
    }

    fn note(&self, name: &str) {
        if !self.ledger.borrow().wants(name) {
            return;
        }

        if let Some(path) = self.resolve(name) {
            self.ledger.borrow_mut().note(name, &path);
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
            .filter_map(|name| stripped(name).map(|(head, extension)| Box::from(format!("{head}.{extension}").as_str())))
            .filter(|name| !files.contains_key(name))
            .collect::<BTreeSet<Box<str>>>()
            .into_iter()
            .collect();

        let localized = bare
            .into_par_iter()
            .filter_map(|name| {
                let path = vfs.variants(&name).into_iter().find_map(|variant| vfs.locate(&variant))?;

                Some((name, path))
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

    fn resolve(&self, name: &str) -> Option<PathBuf> {
        let files = self.files.borrow();

        if let Some(path) = files.get(name) {
            return Some(path.clone());
        }

        let (head, extension) = stripped(name)?;

        files.get(format!("{head}.{extension}").as_str()).cloned()
    }

    fn read(&self, name: &str) -> Option<Vec<u8>> {
        let path = self.resolve(name)?;

        std::fs::read(path).ok()
    }

    fn table(&self, name: &str) -> Option<Vec<u8>> {
        let path = self.resolve(name)?;
        let bytes = std::fs::read(&path).ok()?;
        let piped = path
            .file_name()
            .and_then(|found| found.to_str())
            .is_some_and(|found| Separator::localized(found) == Separator::Pipe);
        let tabular = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension == "csv");

        if !piped || !tabular || !bytes.contains(&b'|') {
            return Some(bytes);
        }

        let mut rewritten = Vec::with_capacity(bytes.len());

        for byte in bytes {
            match byte {
                b',' => rewritten.extend_from_slice(WIDE_COMMA.as_bytes()),
                b'|' => rewritten.push(b','),
                other => rewritten.push(other),
            }
        }

        Some(rewritten)
    }

    fn decode(&mut self, name: &str) -> Option<Sheet> {
        if let Some(sheet) = self.sheets.borrow().get(name) {
            return Some(sheet.clone());
        }

        let decoded = self.read(name).and_then(|bytes| {
            image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
                .inspect_err(|error| warn!("emu: {name} failed to decode: {error}"))
                .ok()
        });
        let sheet = match decoded {
            Some(image) => {
                let mut decoded = image.to_rgba8();

                for pixel in decoded.pixels_mut() {
                    let alpha = u32::from(pixel.0[3]);

                    for channel in &mut pixel.0[..3] {
                        *channel = (u32::from(*channel) * alpha / 0xff) as u8;
                    }
                }

                let (width, height) = (decoded.width(), decoded.height());

                Sheet::new(width, height, decoded.into_raw().as_slice())
            }
            None => return None,
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

        self.note(name);

        let found = self.table(name);

        if found.is_none() && !self.missing.iter().any(|seen| seen.as_ref() == name) {
            debug!("emu: no file named {name}");
            self.missing.push(Box::from(name));
        }

        found
    }

    fn load_png(&mut self, name: &[u8]) -> Option<SheetImage> {
        let name = std::str::from_utf8(name).ok()?;

        self.note(name);

        let sheet = self.decode(name)?;

        Some(SheetImage {
            width: sheet.width as i32,
            height: sheet.height as i32,
            whole: 0,
        })
    }

    fn upload(&mut self, name: &[u8]) {
        if let Ok(name) = std::str::from_utf8(name) {
            self.note(name);
            self.decode(name);
        }
    }

    fn pack_entry(&mut self, _name: &[u8]) -> Vec<u8> {
        Vec::new()
    }
}
