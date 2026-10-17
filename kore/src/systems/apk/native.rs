use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use tracing::{debug, warn};

pub(crate) const LIBRARY: &str = "libnative-lib.so";
const EXTENSION: &str = "so";
const MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
const CLASS_32: u8 = 1;
const CLASS_64: u8 = 2;
const LITTLE_ENDIAN: u8 = 1;
const SHARED_OBJECT: u16 = 3;
const EM_386: u16 = 3;
const EM_ARM: u16 = 40;
const EM_X86_64: u16 = 62;
const EM_AARCH64: u16 = 183;
const HEADER: usize = 20;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum Abi {
    Arm64,
    Arm32,
    X64,
    X86,
}

impl Abi {
    const ALL: [Self; 4] = [Self::Arm64, Self::Arm32, Self::X64, Self::X86];

    pub(crate) fn directory(self) -> &'static str {
        match self {
            Self::Arm64 => "arm64-v8a",
            Self::Arm32 => "armeabi-v7a",
            Self::X64 => "x86_64",
            Self::X86 => "x86",
        }
    }

    pub(crate) fn entry(self) -> String {
        format!("lib/{}/{}", self.directory(), LIBRARY)
    }

    pub(crate) fn from_entry(name: &str) -> Option<Self> {
        let (directory, file) = name.strip_prefix("lib/")?.split_once('/')?;

        if file != LIBRARY {
            return None;
        }

        Self::ALL.into_iter().find(|abi| abi.directory() == directory)
    }

    fn sniff(path: &Path) -> Option<Self> {
        let mut header = [0u8; HEADER];

        File::open(path).and_then(|mut file| file.read_exact(&mut header)).ok()?;
        Self::from_header(&header)
    }

    fn from_header(header: &[u8; HEADER]) -> Option<Self> {
        if header[..4] != MAGIC || header[5] != LITTLE_ENDIAN {
            return None;
        }

        if u16::from_le_bytes([header[16], header[17]]) != SHARED_OBJECT {
            return None;
        }

        match (u16::from_le_bytes([header[18], header[19]]), header[4]) {
            (EM_AARCH64, CLASS_64) => Some(Self::Arm64),
            (EM_ARM, CLASS_32) => Some(Self::Arm32),
            (EM_X86_64, CLASS_64) => Some(Self::X64),
            (EM_386, CLASS_32) => Some(Self::X86),
            _ => None,
        }
    }
}

pub(crate) fn is_library(name: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(EXTENSION))
}

pub(crate) fn resolve(
    candidates: Vec<(String, PathBuf)>,
    shipped: &BTreeSet<Abi>,
    log: &impl Fn(String),
) -> Vec<(Abi, PathBuf)> {
    let mut chosen: BTreeMap<Abi, (String, PathBuf)> = BTreeMap::new();

    for (name, path) in candidates {
        let Some(abi) = Abi::sniff(&path) else {
            warn!(file = %name, path = %path.display(), "Not an Android shared library, skipping");
            log(format!("'{}' is not an Android shared library, skipping.", name));
            continue;
        };

        if !shipped.contains(&abi) {
            warn!(file = %name, abi = abi.directory(), "APK ships no library for this architecture, skipping");
            log(format!("APK has no {} library, skipping '{}'.", abi.directory(), name));
            continue;
        }

        if let Some((kept, _)) = chosen.get(&abi) {
            warn!(file = %name, kept = %kept, abi = abi.directory(), "Duplicate native library for one architecture, keeping the first by name");
            log(format!("'{}' and '{}' are both {} libraries, keeping '{}'.", kept, name, abi.directory(), kept));
            continue;
        }

        debug!(file = %name, abi = abi.directory(), "Mod file replaces a native library");
        log(format!("'{}' replaces the {} native library.", name, abi.directory()));
        chosen.insert(abi, (name, path));
    }

    chosen.into_iter().map(|(abi, (_, path))| (abi, path)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(class: u8, kind: u16, machine: u16) -> [u8; HEADER] {
        let mut bytes = [0u8; HEADER];
        bytes[..4].copy_from_slice(&MAGIC);
        bytes[4] = class;
        bytes[5] = LITTLE_ENDIAN;
        bytes[16..18].copy_from_slice(&kind.to_le_bytes());
        bytes[18..20].copy_from_slice(&machine.to_le_bytes());
        bytes
    }

    #[test]
    fn every_android_abi_is_recognized() {
        assert_eq!(Abi::from_header(&header(CLASS_64, SHARED_OBJECT, EM_AARCH64)), Some(Abi::Arm64));
        assert_eq!(Abi::from_header(&header(CLASS_32, SHARED_OBJECT, EM_ARM)), Some(Abi::Arm32));
        assert_eq!(Abi::from_header(&header(CLASS_64, SHARED_OBJECT, EM_X86_64)), Some(Abi::X64));
        assert_eq!(Abi::from_header(&header(CLASS_32, SHARED_OBJECT, EM_386)), Some(Abi::X86));
    }

    #[test]
    fn executables_and_foreign_files_are_rejected() {
        // A PIE executable is also ET_DYN-adjacent territory, so test the plain ET_EXEC and a bogus magic.
        assert_eq!(Abi::from_header(&header(CLASS_64, 2, EM_AARCH64)), None);
        let mut bogus = header(CLASS_64, SHARED_OBJECT, EM_AARCH64);
        bogus[0] = b'M';
        assert_eq!(Abi::from_header(&bogus), None);
        assert_eq!(Abi::from_header(&header(CLASS_32, SHARED_OBJECT, EM_AARCH64)), None);
    }

    #[test]
    fn entry_paths_round_trip() {
        for abi in Abi::ALL {
            assert_eq!(Abi::from_entry(&abi.entry()), Some(abi));
        }
        assert_eq!(Abi::from_entry("lib/arm64-v8a/libother.so"), None);
        assert_eq!(Abi::from_entry("assets/libnative-lib.so"), None);
    }
}
