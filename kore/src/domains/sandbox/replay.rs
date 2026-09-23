mod save;
mod tape;

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::UNIX_EPOCH;

use rayon::prelude::*;
use tracing::warn;

use crate::common::job::JobOutcome;
use crate::common::{architecture, dirs, solid};
use crate::domains::cat::files;
use crate::domains::cat::scanner::{self, CatEntry};
use crate::domains::settings::ScannerConfig;
use crate::Vault;

pub use save::{God, Level, Options, Parts, Save, Screen, Seeds, Setup, Stage, Unit};
pub use tape::{Cue, Key, format_frame, parse};

pub const EXTENSION: &str = "bcv";
pub const VANILLA_APP: &str = "The Battle Cats";

const SAVE: &str = "save";
const INPUT: &str = "input";
const MANIFEST: &str = "manifest";
const ASSETS: &str = architecture::REPLAY;
const SCRATCH: &str = "replay";
const THEATER: &str = "theater";
const GALLERY: &str = "gallery";
const BASE_FORMS: usize = 2;
const STAMP: &str = "stamp";
const KEEPSAKE_TABLES: [&str; 13] = [
    "unitbuy",
    "unitlevel",
    "SkillAcquisition",
    "SkillLevel",
    "SkillDescriptions",
    "unitevolve",
    "Nyancombo",
    "equipment",
    "gatyaitemD_07_f",
    "uni.png",
    "Skill_name_",
    "img015",
    "img022",
];
const BUNDLE_STEM: &str = "Replay";
const EXPORTS: &str = "exports";
const FORBIDDEN: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

pub fn scratch() -> Option<PathBuf> {
    dirs::state().map(|state| state.join(SCRATCH))
}

pub fn theater() -> Option<PathBuf> {
    dirs::state().map(|state| state.join(THEATER))
}

pub fn gallery() -> Option<PathBuf> {
    dirs::state().map(|state| state.join(GALLERY))
}

pub fn keepsakes(vault: &Vault, units: &[(u32, usize)]) -> Vec<(Box<str>, PathBuf)> {
    let vfs = &vault.vfs;
    let buys = vault.vds.cats.unitbuy(vfs);
    let mut prefixes: Vec<String> = KEEPSAKE_TABLES.iter().map(|table| (*table).to_owned()).collect();

    for &(id, form) in units {
        let number = id + 1;
        let eggs = buys.get(&id).map_or((-1, -1), |row| (row.egg_id_normal, row.egg_id_evolved));

        prefixes.extend([
            format!("unit{number:03}"),
            files::icon_file(id, form, eggs),
            files::anim_base_filename(id, form, eggs),
            format!("Unit_Explanation{number}_"),
        ]);
    }

    prefixes
        .iter()
        .flat_map(|prefix| vfs.glob(prefix))
        .filter_map(|name| vfs.locate(&name).map(|path| (name, path)))
        .collect()
}

fn signature(path: &Path) -> Option<String> {
    let meta = fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_nanos();

    Some(format!("{}:{}:{modified}", path.display(), meta.len()))
}

pub fn stamp(target: &Path) -> Option<String> {
    if !target.is_dir() {
        return signature(target);
    }

    let parts: Option<Vec<String>> = [SAVE, INPUT, MANIFEST].iter().map(|name| signature(&target.join(name))).collect();

    parts.map(|parts| parts.join("|"))
}

pub struct Staged {
    pub summary: Summary,
    pub vault: Vault,
    pub cats: Vec<CatEntry>,
}

pub fn stage(target: &Path, config: &ScannerConfig) -> Result<Staged, String> {
    let bundle = !target.is_dir();
    let dir = if bundle {
        let dir = gallery().ok_or("there is no state folder to unpack the replay into")?;
        let wanted = stamp(target);
        let held = fs::read_to_string(dir.join(STAMP)).ok();

        if wanted.is_none() || held != wanted {
            unpack(target, &dir)?;

            if let Some(wanted) = &wanted
                && let Err(error) = fs::write(dir.join(STAMP), wanted)
            {
                warn!("Replay stamp could not be written: {error}");
            }
        }

        dir
    } else {
        target.to_path_buf()
    };
    let mut summary = inspect_dir(&dir);

    if bundle {
        summary.bytes = fs::metadata(target).map_or(0, |meta| meta.len());
    }

    let vault = Vault::with_priority(&config.language_priority);

    vault
        .vfs
        .create(dir.join(ASSETS).as_path())
        .map_err(|error| format!("the replay's files could not be read: {error}"))?;

    let mut units: Vec<u32> = summary
        .save
        .iter()
        .flat_map(|save| save.setup.lineup.iter())
        .filter_map(|unit| u32::try_from(unit.unit).ok())
        .collect();

    units.dedup();

    let lenient = ScannerConfig { show_invalid_cats: true, ..config.clone() };
    let mut cats: Vec<CatEntry> = units.into_iter().filter_map(|id| scanner::scan_single(id, &vault, &lenient)).collect();

    for unit in summary.save.iter().flat_map(|save| save.setup.lineup.iter()) {
        let fielded = cats
            .iter_mut()
            .find(|cat| i32::try_from(cat.id).is_ok_and(|id| id == unit.unit))
            .zip(usize::try_from(unit.form).ok())
            .and_then(|(cat, form)| cat.forms.get_mut(form));

        if let Some(present) = fielded {
            *present = true;
        }
    }

    for cat in &mut cats {
        for form in 0..BASE_FORMS {
            cat.forms[form] |= cat.stats[form].is_some();
        }

        for form in 0..cat.forms.len() {
            if cat.forms[form] && cat.deploy_icon_paths[form].is_none() {
                cat.deploy_icon_paths[form] = scanner::deploy_icon(&vault.vfs, cat, form);
            }
        }
    }

    Ok(Staged { summary, vault, cats })
}

pub fn library() -> PathBuf {
    PathBuf::from(architecture::SANDBOX)
}

fn stored_name(path: &Path, taken: &BTreeSet<Box<str>>) -> Box<str> {
    let base = path.file_name().and_then(|name| name.to_str()).unwrap_or("asset");

    if !taken.contains(base) {
        return Box::from(base);
    }

    (1usize..)
        .map(|copy| format!("{copy}_{base}"))
        .find(|name| !taken.contains(name.as_str()))
        .map_or_else(|| Box::from(base), Box::from)
}

pub struct Recording {
    dir: PathBuf,
    input: BufWriter<File>,
    manifest: BufWriter<File>,
    kept: BTreeSet<Box<str>>,
    stored: BTreeMap<PathBuf, Box<str>>,
    taken: BTreeSet<Box<str>>,
}

impl Recording {
    pub fn begin(dir: &Path) -> io::Result<Self> {
        if dir.exists() {
            let mut stale = Vec::new();

            walk(dir, "", &mut stale)?;
            stale.par_iter().try_for_each(|(_, path)| fs::remove_file(path))?;
            fs::remove_dir_all(dir)?;
        }

        fs::create_dir_all(dir.join(ASSETS))?;

        Ok(Self {
            dir: dir.to_path_buf(),
            input: BufWriter::new(File::create(dir.join(INPUT))?),
            manifest: BufWriter::new(File::create(dir.join(MANIFEST))?),
            kept: BTreeSet::new(),
            stored: BTreeMap::new(),
            taken: BTreeSet::new(),
        })
    }

    pub fn write_save(&self, save: &Save) -> io::Result<()> {
        let text = serde_json::to_string_pretty(save).map_err(io::Error::other)?;

        fs::write(self.dir.join(SAVE), text)
    }

    pub fn keep(&mut self, requested: Vec<(Box<str>, PathBuf)>) -> Vec<(Box<str>, io::Error)> {
        let mut planned: Vec<(Box<str>, Box<str>)> = Vec::new();
        let mut copies: Vec<(PathBuf, Box<str>)> = Vec::new();

        for (name, path) in requested {
            if !self.kept.insert(name.clone()) {
                continue;
            }

            let stored = match self.stored.get(&path) {
                Some(stored) => stored.clone(),
                None => {
                    let fresh = stored_name(&path, &self.taken);

                    self.taken.insert(fresh.clone());
                    self.stored.insert(path.clone(), fresh.clone());
                    copies.push((path, fresh.clone()));

                    fresh
                }
            };

            planned.push((name, stored));
        }

        let assets = self.dir.join(ASSETS);
        let failed: BTreeMap<Box<str>, io::Error> = copies
            .par_iter()
            .filter_map(|(path, stored)| fs::copy(path, assets.join(stored.as_ref())).err().map(|error| (stored.clone(), error)))
            .collect();
        let mut problems = Vec::new();

        for (name, stored) in planned {
            if let Some(error) = failed.get(&stored) {
                problems.push((name, io::Error::new(error.kind(), error.to_string())));

                continue;
            }

            if let Err(error) = writeln!(self.manifest, "{name}\t{stored}") {
                problems.push((name, error));
            }
        }

        if let Err(error) = self.manifest.flush() {
            problems.push((Box::from(MANIFEST), error));
        }

        problems
    }

    pub fn frame(&mut self, cues: &[Cue]) -> io::Result<()> {
        writeln!(self.input, "{}", format_frame(cues))
    }

    pub fn finish(&mut self) -> io::Result<()> {
        self.input.flush()?;
        self.manifest.flush()
    }
}

fn bundled(name: &str) -> bool {
    name.strip_prefix(ASSETS).is_some_and(|rest| rest.starts_with('/'))
}

pub fn index(dir: &Path) -> io::Result<BTreeMap<Box<str>, PathBuf>> {
    let manifest = BufReader::new(File::open(dir.join(MANIFEST))?);
    let assets = dir.join(ASSETS);
    let mut index = BTreeMap::new();

    for line in manifest.lines() {
        let line = line?;

        if let Some((requested, stored)) = line.split_once('\t') {
            index.insert(Box::from(requested), assets.join(stored));
        }
    }

    Ok(index)
}

pub fn read_save(dir: &Path) -> Result<Save, String> {
    let text = fs::read_to_string(dir.join(SAVE)).map_err(|error| format!("the replay has no readable save: {error}"))?;

    serde_json::from_str(&text).map_err(|error| format!("the replay save is malformed: {error}"))
}

pub fn read_input(dir: &Path) -> Result<Vec<Vec<Cue>>, String> {
    let text = fs::read_to_string(dir.join(INPUT)).map_err(|error| format!("the replay has no readable input: {error}"))?;

    parse(&text)
}

fn walk(dir: &Path, prefix: &str, found: &mut Vec<(String, PathBuf)>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let named = if prefix.is_empty() { name } else { format!("{prefix}/{name}") };

        if path.is_dir() {
            walk(&path, &named, found)?;
        } else {
            found.push((named, path));
        }
    }

    Ok(())
}

pub fn next_bundle(library: &Path) -> PathBuf {
    (1usize..)
        .map(|number| library.join(format!("{BUNDLE_STEM}{number}.{EXTENSION}")))
        .find(|path| !path.exists())
        .unwrap_or_else(|| library.join(format!("{BUNDLE_STEM}.{EXTENSION}")))
}

pub fn export(bundle: &Path) -> Result<PathBuf, String> {
    let folder = Path::new(EXPORTS);
    let stem = bundle.file_stem().map_or_else(|| BUNDLE_STEM.to_owned(), |stem| stem.to_string_lossy().into_owned());
    let out = (0usize..)
        .map(|copy| if copy == 0 { format!("{stem}.{EXTENSION}") } else { format!("{stem}{copy}.{EXTENSION}") })
        .map(|name| folder.join(name))
        .find(|path| !path.exists())
        .unwrap_or_else(|| folder.join(format!("{stem}.{EXTENSION}")));

    fs::create_dir_all(folder).map_err(|error| format!("the exports folder could not be created: {error}"))?;
    fs::copy(bundle, &out).map_err(|error| format!("{} could not be copied: {error}", bundle.display()))?;

    Ok(out)
}

pub fn rename(bundle: &Path, name: &str) -> Result<PathBuf, String> {
    let clean: String = name.chars().filter(|c| !FORBIDDEN.contains(c)).collect();
    let clean = clean.trim();

    if clean.is_empty() {
        return Err("a replay needs a name".to_owned());
    }

    let parent = bundle.parent().ok_or("the replay has no folder")?;
    let target = parent.join(format!("{clean}.{EXTENSION}"));
    let recasing = bundle.file_stem().is_some_and(|stem| stem.to_string_lossy().eq_ignore_ascii_case(clean));

    if !recasing && target.exists() {
        return Err(format!("{clean} is already taken"));
    }

    fs::rename(bundle, &target).map_err(|error| format!("{} could not be renamed: {error}", bundle.display()))?;

    Ok(target)
}

pub fn delete(bundle: &Path) -> Result<(), String> {
    fs::remove_file(bundle).map_err(|error| format!("{} could not be deleted: {error}", bundle.display()))
}

pub fn pack(dir: &Path, out: &Path, emit: impl Fn(f32), abort: &AtomicBool) -> JobOutcome {
    let outcome = match pack_into(dir, out, &emit, abort) {
        Ok(true) => return JobOutcome::Completed,
        Ok(false) => JobOutcome::Aborted,
        Err(reason) => {
            warn!("Replay could not be saved to {}: {reason}", out.display());

            JobOutcome::Failed(reason)
        }
    };

    if out.exists()
        && let Err(error) = fs::remove_file(out)
    {
        warn!("Partial replay {} could not be removed: {error}", out.display());
    }

    outcome
}

fn pack_into(dir: &Path, out: &Path, emit: &dyn Fn(f32), abort: &AtomicBool) -> Result<bool, String> {
    let mut files = Vec::new();

    walk(dir, "", &mut files).map_err(|error| format!("the latest battle could not be read: {error}"))?;

    if !files.iter().any(|(name, _)| name == SAVE) || !files.iter().any(|(name, _)| name == INPUT) {
        return Err("the latest battle has no save or input to keep".to_owned());
    }

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("the replay folder could not be created: {error}"))?;
    }

    let rank = |name: &str| [SAVE, INPUT, MANIFEST].iter().position(|head| *head == name).unwrap_or(usize::MAX);

    files.sort_by(|(left, _), (right, _)| rank(left).cmp(&rank(right)).then_with(|| left.cmp(right)));

    let mut writer = solid::Writer::create(out, solid::DEFAULT_LEVEL).map_err(|error| format!("the replay file could not be created: {error}"))?;

    let sizes: Vec<u64> = files.iter().map(|(_, path)| fs::metadata(path).map_or(0, |meta| meta.len())).collect();
    let total = sizes.iter().sum::<u64>().max(1) as f64;
    let mut packed = 0u64;

    for ((name, path), size) in files.iter().zip(&sizes) {
        if abort.load(Ordering::Relaxed) {
            return Ok(false);
        }

        writer.add(name, path).map_err(|error| format!("{name} could not be added: {error}"))?;
        packed += size;
        emit((packed as f64 / total) as f32);
    }

    writer.finish().map_err(|error| format!("the replay file could not be finished: {error}"))?;

    Ok(!abort.load(Ordering::Relaxed))
}

pub fn unpack(bundle: &Path, dir: &Path) -> Result<(), String> {
    if dir.exists() {
        fs::remove_dir_all(dir).map_err(|error| format!("the previous replay could not be cleared: {error}"))?;
    }

    fs::create_dir_all(dir).map_err(|error| format!("{} could not be created: {error}", dir.display()))?;

    let mut archive = solid::open(bundle).map_err(|error| format!("{} could not be opened: {error}", bundle.display()))?;
    let entries = archive.entries().map_err(|error| format!("{} is not a replay: {error}", bundle.display()))?;

    for entry in entries {
        let mut entry = entry.map_err(|error| format!("{} is damaged: {error}", bundle.display()))?;

        entry.unpack_in(dir).map_err(|error| format!("{} could not be unpacked: {error}", bundle.display()))?;
    }

    Ok(())
}

pub fn restamp(target: &Path, version: &str) -> Result<(), String> {
    let restamped = if target.is_dir() { restamp_dir(target, version) } else { restamp_bundle(target, version) };

    if let Err(reason) = &restamped {
        warn!("Replay {} could not be restamped: {reason}", target.display());
    }

    restamped
}

fn stamped(text: &str, version: &str) -> Result<String, String> {
    let mut save: Save = serde_json::from_str(text).map_err(|error| format!("the replay save is malformed: {error}"))?;

    save.version = version.to_owned();

    serde_json::to_string_pretty(&save).map_err(|error| format!("the replay save could not be written: {error}"))
}

fn restamp_dir(dir: &Path, version: &str) -> Result<(), String> {
    let path = dir.join(SAVE);
    let text = fs::read_to_string(&path).map_err(|error| format!("the replay has no readable save: {error}"))?;

    fs::write(&path, stamped(&text, version)?).map_err(|error| format!("the replay save could not be written: {error}"))
}

fn restamp_bundle(bundle: &Path, version: &str) -> Result<(), String> {
    let partial = bundle.with_extension(format!("{EXTENSION}.part"));
    let rewritten = rewrite_bundle(bundle, &partial, version);

    if rewritten.is_err() && partial.exists() {
        let _ = fs::remove_file(&partial);
    }

    rewritten?;

    fs::rename(&partial, bundle).map_err(|error| format!("{} could not be replaced: {error}", bundle.display()))
}

fn rewrite_bundle(bundle: &Path, partial: &Path, version: &str) -> Result<(), String> {
    let mut archive = solid::open(bundle).map_err(|error| format!("{} could not be opened: {error}", bundle.display()))?;
    let mut writer = solid::Writer::create(partial, solid::DEFAULT_LEVEL).map_err(|error| format!("the replay file could not be created: {error}"))?;
    let entries = archive.entries().map_err(|error| format!("{} is not a replay: {error}", bundle.display()))?;

    for entry in entries {
        let mut entry = entry.map_err(|error| format!("{} is damaged: {error}", bundle.display()))?;
        let name = entry.path().map_err(|error| format!("{} is damaged: {error}", bundle.display()))?.to_string_lossy().into_owned();

        if name == SAVE {
            let mut text = String::new();

            entry.read_to_string(&mut text).map_err(|error| format!("the replay save could not be read: {error}"))?;

            let text = stamped(&text, version)?;

            writer.add_stream(&name, text.len() as u64, text.as_bytes())
        } else {
            let size = entry.size();

            writer.add_stream(&name, size, &mut entry)
        }
        .map_err(|error| format!("{name} could not be rewritten: {error}"))?;
    }

    writer.finish().map_err(|error| format!("the replay file could not be finished: {error}"))
}

#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub save: Option<Save>,
    pub frames: Option<usize>,
    pub assets: usize,
    pub bytes: u64,
}

fn frames_in(text: &str) -> Option<usize> {
    parse(text).ok().map(|frames| frames.len())
}

pub fn inspect_dir(dir: &Path) -> Summary {
    let mut files = Vec::new();

    if walk(dir, "", &mut files).is_err() {
        return Summary::default();
    }

    let bytes = files.iter().filter_map(|(_, path)| fs::metadata(path).ok()).map(|meta| meta.len()).sum();
    let assets = files.iter().filter(|(name, _)| bundled(name)).count();

    Summary {
        save: read_save(dir).ok(),
        frames: fs::read_to_string(dir.join(INPUT)).ok().as_deref().and_then(frames_in),
        assets,
        bytes,
    }
}

struct Head {
    save: Option<String>,
    input: Option<String>,
    assets: usize,
}

fn read_head(bundle: &Path, whole: bool) -> Option<Head> {
    let mut archive = solid::open(bundle).ok()?;
    let mut head = Head { save: None, input: None, assets: 0 };

    for entry in archive.entries().ok()? {
        let mut entry = entry.ok()?;
        let name = entry.path().ok()?.to_string_lossy().into_owned();

        match name.as_str() {
            SAVE | INPUT => {
                let mut text = String::new();

                entry.read_to_string(&mut text).ok()?;

                if name == SAVE {
                    head.save = Some(text);
                } else {
                    head.input = Some(text);
                }
            }
            _ if bundled(&name) => head.assets += 1,
            _ => (),
        }

        if !whole && head.save.is_some() && head.input.is_some() {
            break;
        }
    }

    Some(head)
}

pub fn inspect_bundle(bundle: &Path) -> Summary {
    let bytes = fs::metadata(bundle).map_or(0, |meta| meta.len());
    let Some(head) = read_head(bundle, true) else {
        return Summary { bytes, ..Summary::default() };
    };

    Summary {
        save: head.save.and_then(|text| serde_json::from_str(&text).ok()),
        frames: head.input.as_deref().and_then(frames_in),
        assets: head.assets,
        bytes,
    }
}

fn playable(bundle: &Path) -> bool {
    solid::sniff(bundle) && read_head(bundle, false).is_some_and(|head| head.save.is_some() && head.input.is_some())
}

pub fn list(library: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(library) else {
        return Vec::new();
    };

    let mut found: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case(EXTENSION)))
        .filter(|path| playable(path))
        .collect();

    found.sort_by_key(|path| path.file_name().map(|name| name.to_string_lossy().to_lowercase()));
    found
}
