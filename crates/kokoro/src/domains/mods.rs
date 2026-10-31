#[cfg(feature = "mods")]
mod disk;
#[cfg(feature = "mods")]
pub mod export;
#[cfg(feature = "mods")]
pub mod import;
#[cfg(feature = "bcm")]
mod package;

use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

#[cfg(feature = "mods")]
pub use disk::{ModData, ModDataState, adopt, create, disable, enable, ensure, ensure_as, find, find_all, locate, patch_root, place, reserved, taken};
#[cfg(feature = "bcm")]
pub use package::Package;

pub const METADATA: &str = "metadata.json";

pub const ICON: &str = "icon.png";

pub const PATCH: &str = "patch";

const LOOSE: &str = "loose";
const NESTED: [&str; 3] = ["patch/", "loose/", "icons/"];
const PACK_PARTS: [&str; 2] = ["assets/DownloadLocal.list", "assets/DownloadLocal.pack"];
const ASSETS: &str = "assets";
const JUNK: [&str; 5] = [".dat", ".lock", ".pack", ".list", ".dex"];

pub fn placement(name: &str) -> Option<(String, bool)> {
    let path = Path::new(name);
    let file = path.file_name()?.to_string_lossy().into_owned();

    if NESTED.iter().any(|nested| name.starts_with(nested)) {
        let nested = path.components().all(|part| matches!(part, Component::Normal(_)));

        return nested.then(|| (name.to_owned(), false));
    }

    if PACK_PARTS.contains(&name) {
        return Some((file, true));
    }

    if file == METADATA {
        return Some((format!("{PATCH}/{file}"), false));
    }

    if path.parent() != Some(Path::new(ASSETS)) {
        return None;
    }

    let is_download_tsv = file.starts_with("download") && file.ends_with(".tsv");
    let is_junk = JUNK.iter().any(|junk| file.ends_with(junk));

    (!is_download_tsv && !is_junk).then(|| (format!("{LOOSE}/{file}"), false))
}

fn default_source() -> String {
    "Battle Cats Complete".to_string()
}

fn default_package() -> String {
    "".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMetadata {
    #[serde(default)] pub title: String,
    #[serde(default)] pub author: String,
    #[serde(default)] pub version: String,
    #[serde(default)] pub description: String,
    #[serde(default = "default_package")] pub package: String,
    #[serde(default = "default_source")] pub source: String,
}

impl Default for ModMetadata {
    fn default() -> Self {
        Self {
            title: String::new(),
            author: String::new(),
            version: String::new(),
            description: String::new(),
            package: default_package(),
            source: default_source(),
        }
    }
}

