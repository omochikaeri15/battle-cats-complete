use std::collections::HashMap;
use std::fs;
use std::path::Path;

use tracing::{debug, error, info, trace, warn};

use crate::global::resolver;

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
        self.map.get(key).map(|value_string| value_string.as_str()).unwrap_or("")
    }
}

/// Load the localizable.tsv file using regional fallbacks.
pub fn load_localizable(dir: &Path, priority: &[String]) -> Localizable {
    info!("Initializing localizable dictionary load");
    trace!(directory = %dir.display(), "Attempting to load localizable.tsv");

    // Build the regional filenames based on the priority list
    let mut target_files: Vec<String> = priority
        .iter()
        .map(|language_code| format!("localizable_{}.tsv", language_code))
        .collect();

    // Add the base non-regional file as the ultimate fallback
    target_files.push("localizable.tsv".to_string());

    // Map to &str for the resolver to consume
    let file_targets: Vec<&str> = target_files.iter().map(|target_string| target_string.as_str()).collect();

    // Attempt to grab the file using the dynamically generated regional targets
    let paths = resolver::get(dir, file_targets, priority);

    let Some(file_path) = paths.first() else {
        warn!("Could not find any localizable.tsv file in the given path");
        return Localizable::default();
    };

    debug!(path = %file_path.display(), "Located localizable file, beginning read");

    let Ok(content) = fs::read_to_string(file_path) else {
        error!(path = %file_path.display(), "Found localizable.tsv, but failed to read string data");
        return Localizable::default();
    };

    let mut map = HashMap::new();

    for (line_number, line) in content.lines().enumerate() {
        let clean_line = line.split("//").next().unwrap_or("").trim();
        if clean_line.is_empty() {
            continue;
        }

        // PONOS often mixes tabs and spaces in localizable.tsv.
        // Keys never contain whitespace, so splitting at the first whitespace
        // safely handles both \t and accidental spaces.
        let Some(whitespace_index) = clean_line.find(char::is_whitespace) else {
            trace!(line_number, "Skipping line: no whitespace separator found");
            continue;
        };

        let parsed_key = clean_line[..whitespace_index].trim().to_string();
        let parsed_value = clean_line[whitespace_index..].trim().to_string();

        trace!(key = %parsed_key, "Successfully parsed localization string");
        map.insert(parsed_key, parsed_value);
    }

    info!(count = map.len(), path = %file_path.display(), "Successfully loaded localization strings");

    Localizable { map }
}