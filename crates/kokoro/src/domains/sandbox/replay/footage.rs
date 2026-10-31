use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{Read, Write};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use structured_zstd::decoding::StreamingDecoder;
use structured_zstd::encoding::{CompressionLevel, StreamingEncoder};

use crate::common::counted::Counted;

use super::{ASSETS, Cue, FAULT, Forgiven, INPUT, MANIFEST, SAVE, Save, Summary, bundled, escape, format_frame, frames_in, parse, parse_faults, requests};

const FILE_MODE: u32 = 0o644;
const PACK_LEVEL: i32 = 19;
const PACK_CHUNK: usize = 1 << 20;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Footage {
    pub save: Save,
    pub input: String,
    pub fault: String,
    pub manifest: Vec<(Box<str>, Box<str>)>,
    pub frames: usize,
}

fn append(builder: &mut tar::Builder<Vec<u8>>, name: &str, data: &[u8]) -> Result<(), String> {
    let mut header = tar::Header::new_gnu();

    header.set_size(data.len() as u64);
    header.set_mode(FILE_MODE);
    header.set_mtime(0);
    header.set_cksum();

    builder.append_data(&mut header, name, data).map_err(|error| format!("{name} could not be added: {error}"))
}

impl Footage {
    pub fn new(save: Save) -> Self {
        Self { save, ..Self::default() }
    }

    pub fn frame(&mut self, cues: &[Cue]) {
        self.input.push_str(&format_frame(cues));
        self.input.push('\n');
        self.frames += 1;
    }

    pub fn forgive(&mut self, site: &str, reason: &str) {
        let frame = self.frames.saturating_sub(1);

        self.fault.push_str(&format!("{frame}\t{}\t{}\n", escape(site), escape(reason)));
    }

    pub fn keep(&mut self, requested: &str, stored: &str) {
        self.manifest.push((Box::from(requested), Box::from(stored)));
    }

    pub fn cues(&self) -> Result<Vec<Vec<Cue>>, String> {
        parse(&self.input)
    }

    pub fn faults(&self) -> Vec<Forgiven> {
        parse_faults(&self.fault)
    }

    pub fn stored(&self) -> BTreeSet<&str> {
        self.manifest.iter().map(|(_, stored)| stored.as_ref()).collect()
    }

    pub fn requested(&self) -> impl Iterator<Item = &str> {
        self.manifest.iter().map(|(requested, _)| requested.as_ref())
    }

    pub fn text_size(&self) -> usize {
        let save = serde_json::to_string_pretty(&self.save).map_or(0, |save| save.len());
        let manifest: usize = self.manifest.iter().map(|(requested, stored)| requested.len() + stored.len() + 2).sum();

        save + self.input.len() + self.fault.len() + manifest
    }

    pub fn summary(&self, bytes: u64) -> Summary {
        Summary { save: Some(self.save.clone()), frames: frames_in(&self.input), assets: self.stored().len(), bytes }
    }

    pub fn packer(&self, read: impl Fn(&str) -> Option<Arc<[u8]>>) -> Result<(Packer, Vec<Box<str>>), String> {
        let save = serde_json::to_string_pretty(&self.save).map_err(|error| format!("the replay save could not be written: {error}"))?;
        let manifest: String = self.manifest.iter().map(|(requested, stored)| format!("{requested}\t{stored}\n")).collect();
        let mut builder = tar::Builder::new(Vec::new());
        let mut missing = Vec::new();

        builder.mode(tar::HeaderMode::Deterministic);
        append(&mut builder, SAVE, save.as_bytes())?;
        append(&mut builder, INPUT, self.input.as_bytes())?;
        append(&mut builder, MANIFEST, manifest.as_bytes())?;

        if !self.fault.is_empty() {
            append(&mut builder, FAULT, self.fault.as_bytes())?;
        }

        for stored in self.stored() {
            match read(stored) {
                Some(bytes) => append(&mut builder, &format!("{ASSETS}/{stored}"), &bytes)?,
                None => missing.push(Box::from(stored)),
            }
        }

        let archive = builder.into_inner().map_err(|error| format!("the replay file could not be finished: {error}"))?;

        Ok((Packer::new(archive)?, missing))
    }

    pub fn unpack(bytes: &[u8]) -> Result<Self, String> {
        read(bytes, false).map(|opened| opened.footage)
    }

    pub fn open(bytes: &[u8]) -> Result<Opened, String> {
        read(bytes, true)
    }

    pub fn open_with(bytes: &[u8], report: impl FnMut(f32)) -> Result<Opened, String> {
        read(Counted::new(bytes, report), true)
    }
}

pub struct Packer {
    archive: Vec<u8>,
    fed: usize,
    encoder: StreamingEncoder<Vec<u8>>,
}

impl Packer {
    fn new(archive: Vec<u8>) -> Result<Self, String> {
        let mut encoder = StreamingEncoder::new(Vec::new(), CompressionLevel::from_level(PACK_LEVEL));

        encoder.set_pledged_content_size(archive.len() as u64).map_err(|error| format!("the replay could not be compressed: {error}"))?;

        Ok(Self { archive, fed: 0, encoder })
    }

    pub fn progress(&self) -> f32 {
        if self.archive.is_empty() {
            return 1.0;
        }

        self.fed as f32 / self.archive.len() as f32
    }

    pub fn step(&mut self) -> Result<bool, String> {
        let end = self.fed.saturating_add(PACK_CHUNK).min(self.archive.len());
        let chunk = self.archive.get(self.fed..end).unwrap_or_default();

        self.encoder.write_all(chunk).map_err(|error| format!("the replay could not be compressed: {error}"))?;
        self.fed = end;

        Ok(self.fed < self.archive.len())
    }

    pub fn finish(mut self) -> Result<Vec<u8>, String> {
        while self.step()? {}

        self.encoder.finish().map_err(|error| format!("the replay could not be compressed: {error}"))
    }
}

pub struct Opened {
    pub footage: Footage,
    pub files: BTreeMap<Box<str>, Arc<[u8]>>,
}

impl Opened {
    pub fn by_request(&self) -> HashMap<Box<str>, Arc<[u8]>> {
        self.footage.manifest.iter().filter_map(|(requested, stored)| Some((requested.clone(), Arc::clone(self.files.get(stored)?)))).collect()
    }
}

fn read(source: impl Read, whole: bool) -> Result<Opened, String> {
    let decoder = StreamingDecoder::new(source);
    let mut archive = tar::Archive::new(decoder);
    let (mut save, mut input, mut manifest, mut fault) = (None, None, String::new(), String::new());
    let mut files = BTreeMap::new();

    for entry in archive.entries().map_err(|error| format!("the file is not a replay: {error}"))? {
        let mut entry = entry.map_err(|error| format!("the replay is damaged: {error}"))?;
        let name = entry.path().map_err(|error| format!("the replay is damaged: {error}"))?.to_string_lossy().into_owned();

        if bundled(&name) {
            if !whole && save.is_some() && input.is_some() {
                break;
            }

            if whole && let Some(stored) = name.strip_prefix(ASSETS).and_then(|rest| rest.strip_prefix('/')) {
                let mut held = Vec::new();

                entry.read_to_end(&mut held).map_err(|error| format!("{name} could not be read from the replay: {error}"))?;
                files.insert(Box::from(stored), Arc::from(held));
            }

            continue;
        }

        if ![SAVE, INPUT, MANIFEST, FAULT].contains(&name.as_str()) {
            continue;
        }

        let mut text = String::new();

        entry.read_to_string(&mut text).map_err(|error| format!("{name} could not be read from the replay: {error}"))?;

        match name.as_str() {
            SAVE => save = Some(text),
            INPUT => input = Some(text),
            MANIFEST => manifest = text,
            _ => fault = text,
        }
    }

    let save: Save = serde_json::from_str(&save.ok_or("the replay has no readable save")?).map_err(|error| format!("the replay save is malformed: {error}"))?;
    let input = input.ok_or("the replay has no readable input")?;
    let frames = parse(&input)?.len();
    let manifest = requests(&manifest).map(|(requested, stored)| (Box::from(requested), Box::from(stored))).collect();

    Ok(Opened { footage: Footage { save, input, fault, manifest, frames }, files })
}

#[cfg(test)]
mod tests {
    use super::*;

    // A packed replay has to come back with its save, input, faults and manifest intact.
    #[test]
    fn a_packed_replay_unpacks_to_the_same_footage() {
        let mut footage = Footage::new(Save { version: "1.0".to_owned(), ..Save::default() });

        footage.frame(&[Cue::Press(4, 8), Cue::Release]);
        footage.frame(&[]);
        footage.forgive("main_draw.rs:9", "reached\tthe end");
        footage.keep("img015.png", "img015_en.png");

        let (packer, missing) = footage.packer(|name| (name == "img015_en.png").then(|| Arc::from(&b"png"[..]))).expect("archived");
        let packed = packer.finish().expect("packed");
        let unpacked = Footage::unpack(&packed).expect("unpacked");

        assert!(missing.is_empty());
        assert_eq!(unpacked, footage);

        let opened = Footage::open(&packed).expect("opened");

        assert_eq!(opened.footage, footage);

        let mut reported = Vec::new();
        let counted = Footage::open_with(&packed, |share| reported.push(share)).expect("opened with progress");

        assert_eq!(counted.footage, footage);
        assert_eq!(reported.last().copied(), Some(1.0));
        assert!(reported.windows(2).all(|pair| pair[0] <= pair[1]));
        assert_eq!(opened.by_request().get("img015.png").map(|bytes| bytes.as_ref()), Some(&b"png"[..]));
        assert_eq!(unpacked.faults().len(), 1);
        assert!(Footage::unpack(b"not a replay").is_err());
    }

    // A replay packed in memory has to open with the same reader the desktop uses for its library.
    #[cfg(all(feature = "disk", feature = "mods"))]
    #[test]
    fn a_packed_replay_opens_as_a_desktop_bundle() {
        let mut footage = Footage::new(Save { app: "web".to_owned(), ..Save::default() });

        footage.frame(&[Cue::Press(1, 2)]);
        footage.keep("img015.png", "img015_en.png");

        let packed = footage.packer(|_| Some(Arc::from(&b"png"[..]))).and_then(|(packer, _)| packer.finish()).expect("packed");
        let path = std::env::temp_dir().join(format!("bcc-footage-{}.bcv", std::process::id()));

        std::fs::write(&path, &packed).expect("written");

        let bundle = super::super::Bundle::open(&path);
        let _ = std::fs::remove_file(&path);
        let bundle = bundle.expect("a desktop bundle");

        assert_eq!(bundle.save().map(|save| save.app), Ok("web".to_owned()));
        assert_eq!(bundle.input().map(|frames| frames.len()), Ok(1));
        assert_eq!(bundle.summary().assets, 1);
    }
}
