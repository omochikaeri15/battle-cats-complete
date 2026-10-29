use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use image::RgbaImage;
use codex::chapter::stage::{Background, BackgroundEntry};
use codex::graphics::backdrop::{self, Assets, EffectRig, EffectsFile};
use codex::graphics::rig::{Animation, Model, SpriteSheet};
use tracing::warn;

use crate::Vfs;

pub const TABLE: &str = "bg.csv";
pub const DEFAULT_SEED: u32 = 100;
pub const STAGE_WIDTH: i32 = 4000;
pub const FRAME_RATE: u32 = 30;

const REDIRECT_LIMIT: usize = 8;

#[derive(Default, Clone, PartialEq, Eq)]
pub struct Overrides {
    pub png: Option<PathBuf>,
    pub imgcut: Option<PathBuf>,
    pub csv: Option<PathBuf>,
}

impl Overrides {
    pub fn is_empty(&self) -> bool {
        self.png.is_none() && self.imgcut.is_none() && self.csv.is_none()
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    pub ids: Vec<u32>,
}

impl Catalog {
    pub fn scan(vfs: &Vfs) -> Self {
        let Some(bytes) = vfs.find(TABLE).and_then(|path| vfs.read(&path).ok()) else {
            return Self::default();
        };
        let Ok(table) = Background::parse(bytes.as_ref(), None) else {
            return Self::default();
        };
        let ids: BTreeSet<u32> = table
            .entries
            .iter()
            .filter(|entry| {
                vfs.find(backdrop::sheet_name(entry).as_str()).is_some()
                    && vfs.find(backdrop::cuts_name(entry).as_str()).is_some()
            })
            .map(|entry| entry.id)
            .collect();

        Self { ids: ids.into_iter().collect() }
    }

    pub fn from_table(table: &Background) -> Self {
        let ids: BTreeSet<u32> = table.entries.iter().map(|entry| entry.id).collect();

        Self { ids: ids.into_iter().collect() }
    }

    pub fn ranges(&self) -> String {
        let mut text = String::new();
        let mut run: Option<(u32, u32)> = None;

        for &id in &self.ids {
            match run {
                Some((start, end)) if id == end + 1 => run = Some((start, id)),
                Some(span) => {
                    push_span(&mut text, span);
                    run = Some((id, id));
                }
                None => run = Some((id, id)),
            }
        }

        if let Some(span) = run {
            push_span(&mut text, span);
        }

        text
    }
}

fn push_span(text: &mut String, (start, end): (u32, u32)) {
    if !text.is_empty() {
        text.push_str(", ");
    }

    if start == end {
        let _ = write!(text, "{start}");
    } else {
        let _ = write!(text, "{start}-{end}");
    }
}

#[derive(Clone, Debug)]
pub struct Names {
    pub png: String,
    pub imgcut: String,
    pub csv: String,
}

#[derive(Clone)]
pub struct Loaded {
    pub id: u32,
    pub assets: Assets,
    pub names: Names,
    pub watched: BTreeSet<String>,
}

pub fn export_stem(id: u32) -> String {
    id.to_string()
}

struct Reader<'a> {
    vfs: &'a Vfs,
    overrides: &'a Overrides,
    watched: BTreeSet<String>,
}

impl Reader<'_> {
    fn watch(&mut self, name: &str) {
        self.watched.insert(name.to_ascii_lowercase());
    }

    fn disk(path: &Path) -> Option<Arc<[u8]>> {
        std::fs::read(path)
            .inspect_err(|err| warn!(path = %path.display(), "Background file could not be read: {}", err))
            .ok()
            .map(Arc::from)
    }

    fn mount(&mut self, name: &str) -> Option<Arc<[u8]>> {
        self.watch(name);

        let path = self.vfs.find(name)?;

        self.vfs
            .source(&path)
            .read()
            .inspect_err(|err| warn!(path = %path.display(), "Background file could not be read: {}", err))
            .ok()
    }

    fn name_of(path: &Path) -> String {
        path.file_name().map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().into_owned())
    }

    fn table(&mut self) -> Option<(Arc<[u8]>, String)> {
        match &self.overrides.csv {
            Some(path) => Self::disk(path).map(|bytes| (bytes, Self::name_of(path))),
            None => self.mount(TABLE).map(|bytes| (bytes, TABLE.to_owned())),
        }
    }

    fn sheet(&mut self, entry: &BackgroundEntry) -> Option<(Arc<[u8]>, String)> {
        let name = backdrop::sheet_name(entry);

        match &self.overrides.png {
            Some(path) => Self::disk(path).map(|bytes| (bytes, Self::name_of(path))),
            None => self.mount(&name).map(|bytes| (bytes, name)),
        }
    }

    fn cuts(&mut self, entry: &BackgroundEntry) -> Option<(Arc<[u8]>, String)> {
        let name = backdrop::cuts_name(entry);

        match &self.overrides.imgcut {
            Some(path) => Self::disk(path).map(|bytes| (bytes, Self::name_of(path))),
            None => self.mount(&name).map(|bytes| (bytes, name)),
        }
    }

    fn effects(&mut self, id: u32) -> Vec<backdrop::EffectDef> {
        let mut target = id;

        for _ in 0..REDIRECT_LIMIT {
            let name = backdrop::effects_name(target);
            let Some(bytes) = self.mount(&name) else {
                return Vec::new();
            };

            match EffectsFile::parse(bytes.as_ref()) {
                Ok(EffectsFile::Redirect(next)) => target = next.max(0) as u32,
                Ok(EffectsFile::Defs(defs)) => return defs,
                Err(err) => {
                    warn!(file = name, "Background effect file could not be read: {}", err);
                    return Vec::new();
                }
            }
        }

        Vec::new()
    }

    fn rig_files(&mut self, model: &str, anime: &str) -> Option<(Model, Animation)> {
        let model_bytes = self.mount(model)?;
        let anime_bytes = self.mount(anime)?;
        let model = Model::parse(model_bytes.as_ref())
            .inspect_err(|err| warn!(file = model, "Background effect model could not be parsed: {}", err))
            .ok()?;
        let animation = Animation::parse(anime_bytes.as_ref())
            .inspect_err(|err| warn!(file = anime, "Background effect animation could not be parsed: {}", err))
            .ok()?;

        Some((model, animation))
    }

    fn effect_sheet(&mut self, png: &str, imgcut: &str) -> Option<SpriteSheet> {
        let png_bytes = self.mount(png)?;
        let cut_bytes = self.mount(imgcut)?;

        SpriteSheet::parse(png_bytes.as_ref(), cut_bytes.as_ref())
            .inspect_err(|err| warn!(file = png, "Background effect sheet could not be parsed: {}", err))
            .ok()
    }

    fn image(&mut self, name: &str) -> Option<Arc<RgbaImage>> {
        let bytes = self.mount(name)?;
        let mut image = image::load_from_memory(bytes.as_ref())
            .inspect_err(|err| warn!(file = name, "Background image could not be decoded: {}", err))
            .ok()?
            .to_rgba8();

        for pixel in image.pixels_mut() {
            let alpha = u32::from(pixel[3]);

            for channel in &mut pixel.0[..3] {
                *channel = (u32::from(*channel) * alpha / 255) as u8;
            }
        }

        Some(Arc::new(image))
    }
}

pub fn load(vfs: &Vfs, id: u32, overrides: &Overrides) -> Result<Loaded, String> {
    let mut reader = Reader { vfs, overrides, watched: BTreeSet::new() };

    let (table_bytes, csv_name) = reader.table().ok_or_else(|| format!("{TABLE} is not within the mount"))?;
    let table = Background::parse(table_bytes.as_ref(), None).map_err(|err| err.to_string())?;
    let entry = table.get(id).ok_or_else(|| format!("Background {id} is not listed in {csv_name}"))?.clone();

    let (png_bytes, png_name) = reader.sheet(&entry).ok_or_else(|| format!("{} is not within the mount", backdrop::sheet_name(&entry)))?;
    let (cut_bytes, cut_name) = reader.cuts(&entry).ok_or_else(|| format!("{} is not within the mount", backdrop::cuts_name(&entry)))?;
    let sheet = SpriteSheet::parse(png_bytes.as_ref(), cut_bytes.as_ref()).map_err(|err| err.to_string())?;

    let bubble = backdrop::bubble_name(id).and_then(|name| reader.image(name));
    let sparkle = backdrop::uses_sparkles(id)
        .then(|| reader.effect_sheet(backdrop::SPARKLE_SHEET, backdrop::SPARKLE_CUTS))
        .flatten();

    let defs = reader.effects(id);
    let mut sheets: Vec<(String, SpriteSheet)> = Vec::new();
    let mut files: Vec<Option<EffectRig>> = Vec::with_capacity(defs.len());

    for def in &defs {
        let Some(file) = &def.file else {
            files.push(None);
            continue;
        };

        let sheet_index = match sheets.iter().position(|(name, _)| *name == file.image) {
            Some(index) => Some(index),
            None => reader.effect_sheet(&file.image, &file.imgcut).map(|sheet| {
                sheets.push((file.image.clone(), sheet));
                sheets.len() - 1
            }),
        };

        let rig = sheet_index.zip(reader.rig_files(&file.model, &file.anime)).map(|(index, (model, animation))| EffectRig {
            sheet: Some(index),
            model,
            animation,
        });

        files.push(rig);
    }

    let mut models = Vec::new();

    for index in 0.. {
        let model = backdrop::model_name(&entry, index);
        let anime = backdrop::animation_name(&entry, index);

        let Some((model, animation)) = reader.rig_files(&model, &anime) else {
            break;
        };

        models.push(EffectRig { sheet: None, model, animation });
    }

    let assets = Assets {
        entry,
        sheet,
        bubble,
        sparkle,
        defs,
        sheets: sheets.into_iter().map(|(_, sheet)| sheet).collect(),
        files,
        models,
    };

    Ok(Loaded { id, assets, names: Names { png: png_name, imgcut: cut_name, csv: csv_name }, watched: reader.watched })
}

#[cfg(test)]
mod tests {
    use super::Catalog;

    #[test]
    fn ranges_collapse_runs_and_keep_lone_ids() {
        let catalog = Catalog { ids: vec![0, 1, 2, 3, 5, 100, 101, 102] };

        assert_eq!(catalog.ranges(), "0-3, 5, 100-102");
    }

    #[test]
    fn an_empty_catalog_has_no_ranges() {
        assert_eq!(Catalog::default().ranges(), "");
    }
}


