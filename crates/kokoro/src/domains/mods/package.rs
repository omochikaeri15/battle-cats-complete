use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use structured_zstd::decoding::StreamingDecoder;
use tracing::warn;

use crate::common::counted::Counted;

use super::{METADATA, ModMetadata, placement};

const ZSTD_MAGIC: [u8; 4] = [0x28, 0xb5, 0x2f, 0xfd];

pub struct Package {
    pub metadata: ModMetadata,
    pub files: Vec<(Box<str>, Arc<[u8]>)>,
    pub conflicts: Vec<Box<str>>,
}

impl Package {
    pub fn open(bytes: &[u8], report: impl FnMut(f32)) -> Result<Self, String> {
        if !bytes.starts_with(&ZSTD_MAGIC) {
            return Err("this is not a BCM made by the current desktop app".to_owned());
        }

        let mut archive = tar::Archive::new(StreamingDecoder::new(Counted::new(bytes, report)));
        let mut placed: BTreeMap<String, Arc<[u8]>> = BTreeMap::new();

        for entry in archive.entries().map_err(|error| format!("the file is not a mod: {error}"))? {
            let mut entry = entry.map_err(|error| format!("the mod is damaged: {error}"))?;

            if !entry.header().entry_type().is_file() {
                continue;
            }

            let name = entry.path().map_err(|error| format!("the mod is damaged: {error}"))?.to_string_lossy().replace('\\', "/");
            let Some((relative, pack)) = placement(&name) else {
                continue;
            };

            if pack {
                return Err("this mod carries encrypted game packs, which only the desktop app can import".to_owned());
            }

            let mut held = Vec::new();

            entry.read_to_end(&mut held).map_err(|error| format!("{name} could not be read from the mod: {error}"))?;
            placed.insert(relative, Arc::from(held));
        }

        if placed.is_empty() {
            return Err("the mod holds no files".to_owned());
        }

        let metadata = placed
            .iter()
            .filter(|(relative, _)| Path::new(relative.as_str()).file_name().is_some_and(|name| name == METADATA))
            .min_by_key(|(relative, _)| (Path::new(relative.as_str()).components().count(), relative.as_str()))
            .map_or_else(ModMetadata::default, |(relative, bytes)| {
                serde_json::from_slice(bytes).unwrap_or_else(|error| {
                    warn!("{relative} could not be read, the mod shows no details: {error}");
                    ModMetadata::default()
                })
            });
        let mut named: BTreeMap<Box<str>, Vec<Arc<[u8]>>> = BTreeMap::new();

        for (relative, bytes) in placed {
            if let Some(name) = Path::new(&relative).file_name().map(|name| name.to_string_lossy()) {
                named.entry(Box::from(name.as_ref())).or_default().push(bytes);
            }
        }

        let mut files = Vec::new();
        let mut conflicts = Vec::new();

        for (name, mut copies) in named {
            match (copies.pop(), copies.is_empty()) {
                (Some(bytes), true) => files.push((name, bytes)),
                _ => conflicts.push(name),
            }
        }

        Ok(Self { metadata, files, conflicts })
    }
}

#[cfg(test)]
mod tests {
    use structured_zstd::encoding::{CompressionLevel, compress_slice_to_vec};

    use super::*;

    fn bcm(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());

        for (name, data) in entries {
            let mut header = tar::Header::new_gnu();

            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, name, *data).expect("appended");
        }

        compress_slice_to_vec(&builder.into_inner().expect("archived"), CompressionLevel::from_level(3))
    }

    // A desktop BCM keeps its patch, loose and icon files flat by name, and a name that appears twice is
    // left out entirely, the way the desktop mount excludes every copy.
    #[test]
    fn a_bcm_opens_into_flat_files_with_its_metadata() {
        let packed = bcm(&[
            ("metadata.json", br#"{"title":"Neko","author":"Someone"}"#),
            ("patch/unit001.csv", b"1,2,3"),
            ("loose/img015.png", b"png"),
            ("icons/icon.png", b"icon"),
            ("patch/twice.csv", b"a"),
            ("loose/twice.csv", b"b"),
            ("elsewhere/ignored.csv", b"x"),
        ]);
        let mut reported = Vec::new();
        let package = Package::open(&packed, |share| reported.push(share)).expect("opened");
        let names: Vec<&str> = package.files.iter().map(|(name, _)| name.as_ref()).collect();

        assert_eq!(package.metadata.title, "Neko");
        assert_eq!(package.metadata.author, "Someone");
        assert_eq!(names, ["icon.png", "img015.png", "metadata.json", "unit001.csv"]);
        assert_eq!(package.conflicts, [Box::from("twice.csv")]);
        assert_eq!(reported.last().copied(), Some(1.0));
    }

    // Packs need the game's keys, which the web never holds, and a zip from an older desktop is not a BCM it reads.
    #[test]
    fn encrypted_packs_and_zips_are_refused() {
        assert!(Package::open(&bcm(&[("assets/DownloadLocal.pack", b"secret")]), |_| {}).is_err());
        assert!(Package::open(b"PK\x03\x04", |_| {}).is_err());
    }
}
