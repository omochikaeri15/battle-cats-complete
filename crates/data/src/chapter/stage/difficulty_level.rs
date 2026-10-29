//! Per-stage difficulty ratings shown in the stage selection interface.
use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

/// Represents errors that can occur during the parsing of stage difficulty ratings.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DifficultyLevelError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
}

impl fmt::Display for DifficultyLevelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid difficulty level data."
            ),
        }
    }
}

impl std::error::Error for DifficultyLevelError {}

/// The parsed contents of the stage difficulty rating table.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DifficultyLevel {
    /// The per-stage difficulty ratings in stage order, keyed by map identifier.
    pub map_difficulties: HashMap<u32, Vec<u16>>,
}

impl DifficultyLevel {
    /// Parses the stage difficulty rating table into per-map rating lists.
    ///
    /// Each row lists one map's ratings in stage order, so an entry's position
    /// within the returned vector is its stage index.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the difficulty level file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed `DifficultyLevel` on success, or a
    /// `DifficultyLevelError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, DifficultyLevelError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<DifficultyLevel, DifficultyLevelError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::resolve(separator, &file_content);

    let mut map_difficulties = HashMap::new();
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

        let Some(map_id_str) = parts.first() else { continue; };
        let Ok(map_id) = map_id_str.trim().parse::<u32>() else { continue; };

        let mut difficulties = Vec::new();
        for diff_str in parts.iter().skip(1) {
            let trimmed_diff = diff_str.trim();

            let mut integer_part = trimmed_diff;
            if let Some((before_dot, _)) = trimmed_diff.split_once('.') {
                integer_part = before_dot;
            }

            let parsed_difficulty = integer_part.parse::<u16>().unwrap_or(0);

            difficulties.push(parsed_difficulty);
        }

        map_difficulties.insert(map_id, difficulties);
    }

    if !has_content {
        return Err(DifficultyLevelError::EmptyFile);
    }

    Ok(DifficultyLevel { map_difficulties })
}