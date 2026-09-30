//! Localized display names for maps.
use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

/// Represents errors that can occur during the parsing of localized map names.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MapNameError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
}

impl fmt::Display for MapNameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid map name data."
            ),
        }
    }
}

impl std::error::Error for MapNameError {}

/// The parsed contents of the localized map name table.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapName {
    /// The display names, keyed by map identifier.
    pub names: HashMap<u32, String>,
}

impl MapName {
    /// Parses the localized map name table into names keyed by map identifier.
    ///
    /// Trailing comment text introduced by a double slash is discarded before
    /// the columns are read.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the map name file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed `MapName` on success, or a
    /// `MapNameError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, MapNameError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<MapName, MapNameError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::resolve(separator, &file_content);

    let mut names = HashMap::new();
    let mut has_content = false;

    for file_line in file_content.lines() {
        let mut clean_line = file_line;

        if let Some((before_comment, _)) = file_line.split_once("//") {
            clean_line = before_comment;
        }

        let trimmed_line = clean_line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        has_content = true;

        let parts: Vec<&str> = trimmed_line.split(separator_char).collect();
        if parts.len() < 2 {
            continue;
        }

        let Some(id_string) = parts.first() else { continue; };
        let Ok(map_id) = id_string.trim().parse::<u32>() else { continue; };

        let Some(name_string) = parts.get(1) else { continue; };
        let map_name = name_string.trim();

        if !map_name.is_empty() {
            names.insert(map_id, map_name.to_string());
        }
    }

    if !has_content {
        return Err(MapNameError::EmptyFile);
    }

    Ok(MapName { names })
}