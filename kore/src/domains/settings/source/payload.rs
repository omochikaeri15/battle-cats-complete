use std::fs;
use std::io::{self, Read};
use std::path::Path;

use serde::Deserialize;
use zstd::stream::read::Decoder;

const FORMAT: u32 = 1;
const MARK: [u8; 4] = *b"BGL1";

#[derive(Deserialize, Clone, Debug)]
pub(super) struct Manifest {
    format: u32,
    pub(super) id: String,
    files: u64,
    pub(super) packed: u64,
    pub(super) chunks: Vec<Chunk>,
}

#[derive(Deserialize, Clone, Debug)]
pub(super) struct Chunk {
    pub(super) name: String,
    #[serde(default)]
    pub(super) address: String,
    pub(super) size: u64,
    pub(super) files: u64,
}

#[derive(Debug)]
pub(super) enum Fault {
    Stream(io::Error),
    Disk(io::Error),
}

struct Record {
    size: u64,
    names: Vec<String>,
}

impl Manifest {
    pub(super) fn read(bytes: &[u8]) -> Result<Self, String> {
        let manifest: Self = serde_json::from_slice(bytes).map_err(|err| format!("the manifest could not be read: {}", err))?;

        if manifest.format != FORMAT {
            return Err(format!("the payload is format {}, which this version cannot read", manifest.format));
        }

        if manifest.id.is_empty() || manifest.chunks.is_empty() {
            return Err("the manifest lists no payload".to_owned());
        }

        if let Some(chunk) = manifest.chunks.iter().find(|chunk| !addressed(&chunk.address)) {
            return Err(format!("{} was never published", chunk.name));
        }

        if manifest.chunks.iter().map(|chunk| chunk.size).sum::<u64>() != manifest.packed
            || manifest.chunks.iter().map(|chunk| chunk.files).sum::<u64>() != manifest.files
        {
            return Err("the manifest does not add up to its own totals".to_owned());
        }

        Ok(manifest)
    }
}

pub(super) fn unpack(source: impl Read, staged: &Path) -> Result<Vec<(String, u64)>, Fault> {
    let mut decoder = Decoder::new(source).map_err(Fault::Stream)?;
    let records = records(&mut decoder).map_err(Fault::Stream)?;
    let mut data = Vec::new();
    let mut files = Vec::new();

    for record in records {
        data.clear();
        (&mut decoder).take(record.size).read_to_end(&mut data).map_err(Fault::Stream)?;

        if data.len() as u64 != record.size {
            return Err(Fault::Stream(io::Error::new(io::ErrorKind::UnexpectedEof, "the chunk ends before its last file does")));
        }

        for name in record.names {
            fs::write(staged.join(&name), &data).map_err(Fault::Disk)?;
            files.push((name, record.size));
        }
    }

    if decoder.read(&mut [0u8; 1]).map_err(Fault::Stream)? != 0 {
        return Err(Fault::Stream(invalid("the chunk holds data past its last file")));
    }

    Ok(files)
}

fn records(reader: &mut impl Read) -> io::Result<Vec<Record>> {
    if array::<4>(reader)? != MARK {
        return Err(invalid("the chunk does not start with the payload mark"));
    }

    let count = u32::from_le_bytes(array(reader)?);
    let mut records = Vec::new();

    for _ in 0..count {
        let size = u64::from_le_bytes(array(reader)?);
        let aliases = u32::from_le_bytes(array(reader)?);
        let mut names = Vec::new();

        for _ in 0..aliases {
            let mut name = vec![0u8; usize::from(u16::from_le_bytes(array(reader)?))];

            reader.read_exact(&mut name)?;
            names.push(
                String::from_utf8(name)
                    .ok()
                    .filter(|name| flat(name))
                    .ok_or_else(|| invalid("the chunk names a file outside the folder"))?,
            );
        }

        records.push(Record { size, names });
    }

    Ok(records)
}

fn addressed(address: &str) -> bool {
    !address.is_empty() && address.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
}

fn flat(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\\', ':', '\0'])
}

fn array<const N: usize>(reader: &mut impl Read) -> io::Result<[u8; N]> {
    let mut bytes = [0u8; N];

    reader.read_exact(&mut bytes)?;

    Ok(bytes)
}

fn invalid(reason: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn folder(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("bcc-source-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&folder);

        fs::create_dir_all(&folder).unwrap();

        folder
    }

    fn chunk(blobs: &[(&[&str], &[u8])]) -> Vec<u8> {
        let mut raw = MARK.to_vec();

        raw.extend_from_slice(&(blobs.len() as u32).to_le_bytes());

        for (names, data) in blobs {
            raw.extend_from_slice(&(data.len() as u64).to_le_bytes());
            raw.extend_from_slice(&(names.len() as u32).to_le_bytes());

            for name in *names {
                raw.extend_from_slice(&(name.len() as u16).to_le_bytes());
                raw.extend_from_slice(name.as_bytes());
            }
        }

        for (_, data) in blobs {
            raw.extend_from_slice(data);
        }

        zstd::encode_all(&raw[..], 3).unwrap()
    }

    // One stored blob can stand for several identical files, and each name has to get its own copy.
    #[test]
    fn a_chunk_lands_every_name_it_lists() {
        let staged = folder("lands");
        let packed = chunk(&[(&["unit001.csv", "unit002.csv"], b"1,2,3\n"), (&["000_f.png"], &[7u8; 5000]), (&["empty.tsv"], b"")]);

        let landed = unpack(&packed[..], &staged).unwrap();

        assert_eq!(landed.len(), 4);
        assert_eq!(landed[1], ("unit002.csv".to_owned(), 6));
        assert_eq!(fs::read(staged.join("unit001.csv")).unwrap(), b"1,2,3\n");
        assert_eq!(fs::read(staged.join("unit002.csv")).unwrap(), b"1,2,3\n");
        assert_eq!(fs::read(staged.join("000_f.png")).unwrap(), vec![7u8; 5000]);
        assert!(fs::read(staged.join("empty.tsv")).unwrap().is_empty());

        let _ = fs::remove_dir_all(&staged);
    }

    // The payload comes off the internet, so a name must never be able to climb out of the staging folder.
    #[test]
    fn a_name_that_leaves_the_folder_is_refused_before_anything_is_written() {
        let root = folder("escape");
        let staged = root.join("staged");

        fs::create_dir_all(&staged).unwrap();

        for name in ["../escaped.csv", "cats/unit.csv", "..", "C:evil.csv", ""] {
            let packed = chunk(&[(&["fine.csv"], b"ok"), (&[name], b"bad")]);

            assert!(matches!(unpack(&packed[..], &staged), Err(Fault::Stream(_))), "{name:?} slipped through");
        }

        assert_eq!(fs::read_dir(&staged).unwrap().count(), 0);
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_chunk_cut_short_or_padded_is_a_stream_fault() {
        let staged = folder("cut");
        let packed = chunk(&[(&["000_f.png"], &[9u8; 70_000])]);
        let mut padded = chunk(&[(&["a.csv"], b"1")]);

        padded.extend_from_slice(&zstd::encode_all(&b"extra"[..], 3).unwrap());

        assert!(matches!(unpack(&packed[..packed.len() / 2], &staged), Err(Fault::Stream(_))));
        assert!(matches!(unpack(&padded[..], &staged), Err(Fault::Stream(_))));
        assert!(matches!(unpack(&b"not a chunk at all"[..], &staged), Err(Fault::Stream(_))));

        let _ = fs::remove_dir_all(&staged);
    }

    #[test]
    fn a_manifest_is_only_trusted_once_it_is_published_and_adds_up() {
        let manifest = |format: u32, address: &str, packed: u64| {
            format!(
                r#"{{"format":{format},"id":"abc","files":3,"size":999,"packed":{packed},"chunks":[
                    {{"name":"000.chunk","address":"1AbC_d-9","size":40,"raw":90,"files":2,"hash":"x"}},
                    {{"name":"001.chunk"{address},"size":60,"raw":70,"files":1,"hash":"y"}}]}}"#
            )
        };

        let read = Manifest::read(manifest(1, r#","address":"2xYz""#, 100).as_bytes()).unwrap();

        assert_eq!((read.id.as_str(), read.packed, read.chunks.len()), ("abc", 100, 2));
        assert_eq!(read.chunks[1].address, "2xYz");

        assert!(Manifest::read(manifest(2, r#","address":"2xYz""#, 100).as_bytes()).is_err());
        assert!(Manifest::read(manifest(1, "", 100).as_bytes()).is_err());
        assert!(Manifest::read(manifest(1, r#","address":"2x/../Yz""#, 100).as_bytes()).is_err());
        assert!(Manifest::read(manifest(1, r#","address":"2xYz""#, 101).as_bytes()).is_err());
        assert!(Manifest::read(b"<html>Sign in</html>").is_err());
    }
}
