mod save;
mod tape;

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use tracing::warn;

use crate::common::{architecture, dirs, solid};

pub use save::{God, Level, Options, Parts, Save, Screen, Seeds, Setup, Stage, Unit};
pub use tape::{Cue, Key, format_frame, parse};

pub const EXTENSION: &str = "bcv";

const SAVE: &str = "save";
const INPUT: &str = "input";
const MANIFEST: &str = "manifest";
const ASSETS: &str = "assets";
const SCRATCH: &str = "replay";
const THEATER: &str = "theater";
const BUNDLE_STEM: &str = "Replay";

pub fn scratch() -> Option<PathBuf> {
    dirs::state().map(|state| state.join(SCRATCH))
}

pub fn theater() -> Option<PathBuf> {
    dirs::state().map(|state| state.join(THEATER))
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
        .map(|number| library.join(format!("{BUNDLE_STEM} {number}.{EXTENSION}")))
        .find(|path| !path.exists())
        .unwrap_or_else(|| library.join(format!("{BUNDLE_STEM}.{EXTENSION}")))
}

pub fn pack(dir: &Path, out: &Path) -> Result<(), String> {
    let packed = pack_into(dir, out);

    if let Err(reason) = &packed {
        warn!("Replay could not be saved to {}: {reason}", out.display());

        if out.exists()
            && let Err(error) = fs::remove_file(out)
        {
            warn!("Partial replay {} could not be removed: {error}", out.display());
        }
    }

    packed
}

fn pack_into(dir: &Path, out: &Path) -> Result<(), String> {
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

    for (name, path) in &files {
        writer.add(name, path).map_err(|error| format!("{name} could not be added: {error}"))?;
    }

    writer.finish().map_err(|error| format!("the replay file could not be finished: {error}"))
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
    let assets = files.iter().filter(|(name, _)| name.starts_with(ASSETS)).count();

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
            _ if name.starts_with(ASSETS) => head.assets += 1,
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
