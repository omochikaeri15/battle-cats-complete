use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use tracing::{debug, warn};

use codex::combat::Separator;
use emulator::engine::{AssetSource, SheetImage};
use emulator::runtime::HostLimits;
use kokoro::common::io::APP_LANGUAGES;

const UNIT_STEM: &str = "unit";
const TABLE_SUFFIX: &str = ".csv";
const ENEMY_TABLE: &str = "t_unit.csv";
const TALENT_TABLE: &str = "SkillAcquisition.csv";
const TALENT_LEAD: usize = 2;
const TALENT_GROUP: usize = 14;
const EX_MAP_STEM: &str = "MapStageDataRE_";
const CUT_SUFFIX: &str = ".imgcut";
const SHEET_SUFFIX: &str = ".png";
const CUT_HEADER: usize = 4;
const TABLE_KINDS: [&str; 2] = ["csv", "tsv"];
const KEYED_TABLES: [&str; 1] = ["localizable"];
const CLOSERS: [&[u8]; 2] = [b"@", "\u{ff20}".as_bytes()];
const NONE_CODE: &str = "--";

pub const WIDE_COMMA: &str = "\u{ff0c}";

pub trait Files {
    fn read(&self, name: &str) -> Option<Vec<u8>>;
    fn contains(&self, name: &str) -> bool;
    fn names(&self) -> Vec<Box<str>>;
}

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

pub type SheetCache = BTreeMap<Box<str>, Sheet>;

fn cut_rects(bytes: &[u8]) -> Vec<[u32; 4]> {
    String::from_utf8_lossy(bytes)
        .lines()
        .skip(CUT_HEADER)
        .filter_map(|line| {
            let mut cells = line.split(',').map(|cell| cell.trim().parse::<u32>().ok());

            Some([cells.next()??, cells.next()??, cells.next()??, cells.next()??])
        })
        .collect()
}

fn table_rows(bytes: &[u8]) -> (Vec<&[u8]>, Option<&[u8]>) {
    let mut rows: Vec<&[u8]> = bytes.split(|&byte| byte == b'\n').map(|row| row.strip_suffix(b"\r").unwrap_or(row)).collect();

    while rows.last().is_some_and(|row| row.is_empty()) {
        rows.pop();
    }

    let closer = rows.pop_if(|row| CLOSERS.contains(row));

    (rows, closer)
}

fn blank_row(row: &[u8]) -> bool {
    row.iter().all(|byte| byte.is_ascii_whitespace() || *byte == b'|' || *byte == b',')
}

fn row_key(row: &[u8]) -> &[u8] {
    row.split(|&byte| byte == b'\t').next().unwrap_or(row)
}

fn stripped(name: &str) -> Option<(&str, &str)> {
    let (stem, extension) = name.rsplit_once('.')?;
    let (head, code) = stem.rsplit_once('_')?;

    APP_LANGUAGES.iter().any(|&(language, _)| language == code).then_some((head, extension))
}

pub struct Assets {
    files: Rc<dyn Files>,
    sheets: Rc<RefCell<SheetCache>>,
    priority: Vec<String>,
    aliases: RefCell<BTreeMap<Box<str>, Option<Box<str>>>>,
    missing: Vec<Box<str>>,
}

impl Assets {
    pub fn new(files: Rc<dyn Files>, sheets: Rc<RefCell<SheetCache>>, priority: &[String]) -> Self {
        Self { files, sheets, priority: priority.to_vec(), aliases: RefCell::new(BTreeMap::new()), missing: Vec::new() }
    }

    pub fn limits(&self) -> HostLimits {
        let names = self.files.names();
        let units = names.iter().filter_map(|name| name.strip_prefix(UNIT_STEM)?.strip_suffix(TABLE_SUFFIX)?.parse::<usize>().ok()).max().unwrap_or(0);
        let enemy_rows = self.table(ENEMY_TABLE).map_or(0, |bytes| bytes.split(|&byte| byte == b'\n').filter(|row| !row.is_empty()).count());
        let talent_groups = self.table(TALENT_TABLE).map_or(0, |bytes| {
            bytes.split(|&byte| byte == b'\n').map(|row| row.split(|&byte| byte == b',').count().saturating_sub(TALENT_LEAD) / TALENT_GROUP).max().unwrap_or(0)
        });
        let ex_maps = names
            .iter()
            .filter_map(|name| name.strip_prefix(EX_MAP_STEM)?.strip_suffix(TABLE_SUFFIX)?.parse::<usize>().ok())
            .max()
            .map_or(0, |last| last + 1);

        HostLimits { units, enemy_rows, talent_groups, ex_maps, neg5_maps: 0 }
    }

    pub fn resolved(&self) -> usize {
        self.files.names().len()
    }

    fn localized(&self, head: &str, extension: &str) -> Option<Box<str>> {
        let key: Box<str> = Box::from(format!("{head}.{extension}").as_str());

        if let Some(held) = self.aliases.borrow().get(&key) {
            return held.clone();
        }

        let found = self
            .priority
            .iter()
            .filter(|code| code.as_str() != NONE_CODE)
            .map(|code| format!("{head}_{code}.{extension}"))
            .chain(APP_LANGUAGES.iter().map(|&(code, _)| format!("{head}_{code}.{extension}")))
            .find(|variant| self.files.contains(variant))
            .map(|variant| Box::from(variant.as_str()));

        self.aliases.borrow_mut().insert(key, found.clone());

        found
    }

    fn resolve(&self, name: &str) -> Option<Box<str>> {
        if self.files.contains(name) {
            return Some(Box::from(name));
        }

        if let Some((head, extension)) = stripped(name) {
            let bare = format!("{head}.{extension}");

            if self.files.contains(&bare) {
                return Some(Box::from(bare.as_str()));
            }

            return self.localized(head, extension);
        }

        let (stem, extension) = name.rsplit_once('.')?;

        self.localized(stem, extension)
    }

    fn read(&self, name: &str) -> Option<Vec<u8>> {
        self.files.read(&self.resolve(name)?)
    }

    fn fuller(&self, name: &str) -> Option<(Box<str>, Vec<[u32; 4]>)> {
        let stem = name.strip_suffix(CUT_SUFFIX)?;
        let head = stripped(name).map_or(stem, |(head, _)| head);
        let own = cut_rects(&self.read(name)?);

        APP_LANGUAGES
            .iter()
            .filter_map(|&(code, _)| {
                let variant = format!("{head}_{code}");
                let file = format!("{variant}{CUT_SUFFIX}");

                if !self.files.contains(&file) {
                    return None;
                }

                let rects = cut_rects(&self.files.read(&file)?);

                (rects.len() > own.len() && rects.starts_with(&own)).then(|| (Box::from(variant.as_str()), rects.split_at(own.len()).1.to_vec()))
            })
            .max_by_key(|(_, extra)| extra.len())
    }

    fn plain(&self, name: &str) -> Option<Vec<u8>> {
        let resolved = self.resolve(name)?;
        let bytes = self.files.read(&resolved)?;
        let piped = Separator::localized(&resolved) == Separator::Pipe;
        let tabular = resolved.rsplit_once('.').is_some_and(|(_, extension)| extension == "csv");

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

    fn backfill(&self, name: &str, own: Vec<u8>) -> Vec<u8> {
        let Some((head, extension)) = stripped(name).filter(|(_, extension)| TABLE_KINDS.contains(extension)) else {
            return own;
        };

        if KEYED_TABLES.contains(&head) {
            return self.keyed(name, head, extension, own);
        }

        let (rows, closer) = table_rows(&own);
        let donors: Vec<(String, Vec<u8>)> = APP_LANGUAGES
            .iter()
            .filter_map(|&(code, _)| {
                let variant = format!("{head}_{code}.{extension}");

                if variant == name || !self.files.contains(&variant) {
                    return None;
                }

                self.plain(&variant).map(|bytes| (variant, bytes))
            })
            .collect();
        let donor_rows: Vec<(&str, Vec<&[u8]>)> = donors.iter().map(|(variant, bytes)| (variant.as_str(), table_rows(bytes).0)).collect();
        let longest = donor_rows.iter().map(|(_, held)| held.len()).max().unwrap_or(0).max(rows.len());
        let mut borrowed: Vec<&str> = Vec::new();
        let mut filled: Vec<u8> = Vec::with_capacity(own.len());

        for at in 0..longest {
            let held = rows.get(at).copied();
            let row = if held.is_some_and(|row| !blank_row(row)) {
                held
            } else {
                donor_rows
                    .iter()
                    .find_map(|(variant, held)| {
                        let row = held.get(at).copied().filter(|row| !blank_row(row))?;

                        if !borrowed.contains(variant) {
                            borrowed.push(variant);
                        }

                        Some(row)
                    })
                    .or(held)
                    .or_else(|| donor_rows.iter().find_map(|(_, held)| held.get(at).copied()))
            };
            let Some(row) = row else {
                continue;
            };

            if at > 0 {
                filled.push(b'\n');
            }

            filled.extend_from_slice(row);
        }

        if borrowed.is_empty() && longest == rows.len() {
            return own;
        }

        if let Some(closer) = closer {
            filled.push(b'\n');
            filled.extend_from_slice(closer);
        }

        filled.push(b'\n');
        filled
    }

    fn keyed(&self, name: &str, head: &str, extension: &str, own: Vec<u8>) -> Vec<u8> {
        let (rows, closer) = table_rows(&own);
        let mut held: BTreeSet<Vec<u8>> = rows.iter().map(|row| row_key(row).to_vec()).collect();
        let mut surplus: Vec<u8> = Vec::new();

        for &(code, _) in APP_LANGUAGES {
            let variant = format!("{head}_{code}.{extension}");

            if variant == name || !self.files.contains(&variant) {
                continue;
            }

            let Some(bytes) = self.plain(&variant) else {
                continue;
            };

            for row in table_rows(&bytes).0 {
                if !row.is_empty() && held.insert(row_key(row).to_vec()) {
                    surplus.push(b'\n');
                    surplus.extend_from_slice(row);
                }
            }
        }

        if surplus.is_empty() {
            return own;
        }

        let mut filled: Vec<u8> = rows.join(&b'\n');

        filled.append(&mut surplus);

        if let Some(closer) = closer {
            filled.push(b'\n');
            filled.extend_from_slice(closer);
        }

        filled.push(b'\n');
        filled
    }

    fn table(&self, name: &str) -> Option<Vec<u8>> {
        if let Some((variant, _)) = self.fuller(name) {
            return self.files.read(&format!("{variant}{CUT_SUFFIX}"));
        }

        self.plain(name).map(|own| self.backfill(name, own))
    }

    fn premultiply(pixel: &mut [u8; 4]) {
        let alpha = u32::from(pixel[3]);

        for channel in &mut pixel[..3] {
            *channel = (u32::from(*channel) * alpha / 0xff) as u8;
        }
    }

    fn decode(&mut self, name: &str) -> Option<Sheet> {
        if let Some(sheet) = self.sheets.borrow().get(name) {
            return Some(sheet.clone());
        }

        let image = self.read(name).and_then(|bytes| {
            image::load_from_memory_with_format(&bytes, image::ImageFormat::Png).inspect_err(|error| warn!("emu: {name} failed to decode: {error}")).ok()
        })?;
        let mut decoded = image.to_rgba8();

        for pixel in decoded.pixels_mut() {
            Self::premultiply(&mut pixel.0);
        }

        let (width, height) = (decoded.width(), decoded.height());
        let patch = name.strip_suffix(SHEET_SUFFIX).and_then(|stem| self.fuller(&format!("{stem}{CUT_SUFFIX}"))).and_then(|(variant, extra)| {
            let bytes = self.files.read(&format!("{variant}{SHEET_SUFFIX}"))?;
            let donor = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png).ok()?.to_rgba8();

            (donor.width() == width && donor.height() == height).then_some((donor, extra))
        });

        if let Some((donor, extra)) = patch {
            for [left, top, wide, tall] in extra {
                for y in top..top.saturating_add(tall).min(height) {
                    for x in left..left.saturating_add(wide).min(width) {
                        let mut pixel = *donor.get_pixel(x, y);

                        Self::premultiply(&mut pixel.0);
                        decoded.put_pixel(x, y, pixel);
                    }
                }
            }
        }

        let sheet = Sheet::new(width, height, decoded.into_raw().as_slice());

        self.sheets.borrow_mut().insert(Box::from(name), sheet.clone());

        Some(sheet)
    }
}

impl AssetSource for Assets {
    fn open(&mut self, name: &[u8], _packed: u8, _encrypted: u8) -> Option<Vec<u8>> {
        let name = std::str::from_utf8(name).ok()?;
        let found = self.table(name);

        if found.is_none() && !self.missing.iter().any(|seen| seen.as_ref() == name) {
            debug!("emu: no file named {name}");
            self.missing.push(Box::from(name));
        }

        found
    }

    fn load_png(&mut self, name: &[u8]) -> Option<SheetImage> {
        let name = std::str::from_utf8(name).ok()?;
        let sheet = self.decode(name)?;

        Some(SheetImage { width: sheet.width as i32, height: sheet.height as i32, whole: 0 })
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
