use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, trace, warn};

use crate::global::resolver;
use crate::global::utils::detect_csv_separator;

/// A simple lookup table for strings in localizable.tsv
#[derive(Default, Debug, Clone)]
pub struct Localizable {
    map: HashMap<String, String>,
}

impl Localizable {
    /// Look up a translation by its key.
    pub fn lookup(&self, key: &str) -> Option<&String> {
        self.map.get(key)
    }

    /// Look up a translation, returning an empty string if missing.
    pub fn lookup_or_empty(&self, key: &str) -> &str {
        self.map.get(key).map(|s| s.as_str()).unwrap_or("")
    }
}

/// Load the localizable.tsv file using regional fallbacks.
pub fn load_localizable(dir: &Path, priority: &[String]) -> Localizable {
    trace!("Attempting to load localizable.tsv from {}", dir.display());

    // Build the regional filenames based on the priority list
    let mut target_files: Vec<String> = priority
        .iter()
        .map(|lang| format!("localizable_{}.tsv", lang))
        .collect();

    // Add the base non-regional file as the ultimate fallback
    target_files.push("localizable.tsv".to_string());

    // Map to &str for the resolver to consume
    let file_targets: Vec<&str> = target_files.iter().map(|s| s.as_str()).collect();

    // Attempt to grab the file using the dynamically generated regional targets
    let paths = resolver::get(dir, file_targets, priority);

    let Some(file_path) = paths.first() else {
        warn!("Could not find any localizable.tsv file in the given path");
        return Localizable::default();
    };

    let Ok(content) = fs::read_to_string(file_path) else {
        warn!("Found localizable.tsv at {}, but failed to read string data", file_path.display());
        return Localizable::default();
    };

    let sep = detect_csv_separator(&content);
    let mut map = HashMap::new();

    for line in content.lines() {
        let clean = line.split("//").next().unwrap_or("").trim();
        if clean.is_empty() {
            continue;
        }

        let parts: Vec<&str> = clean.split(sep).collect();
        if parts.len() < 2 {
            continue;
        }

        let key = parts[0].trim().to_string();
        let value = parts[1].trim().to_string();

        map.insert(key, value);
    }

    debug!("Loaded {} localization strings from {}", map.len(), file_path.display());

    Localizable { map }
}