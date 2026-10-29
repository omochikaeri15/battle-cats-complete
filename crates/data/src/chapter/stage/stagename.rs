//! Localized display names for individual stages.
use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

/// Represents errors that can occur during the parsing of localized stage names.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StageNameError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
}

impl fmt::Display for StageNameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid stage name data."
            ),
        }
    }
}

impl std::error::Error for StageNameError {}

/// The stage names belonging to a single map.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageNameEntry {
    /// The display names in stage order, so a name's position is its stage index.
    pub names: Vec<String>,
}

/// The parsed contents of the localized stage name table.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageName {
    /// The stage name lists, keyed by map identifier.
    pub entries: HashMap<u32, StageNameEntry>,
}

impl StageName {
    /// Parses the localized stage name table into per-map name lists.
    ///
    /// # Arguments
    /// * `file_bytes` - The raw, decrypted byte slice of the stage name file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed `StageName` on success, or a
    /// `StageNameError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(file_bytes: B, separator: Option<Separator>) -> Result<Self, StageNameError> {
        parse_inner(file_bytes.as_ref(), separator)
    }
}

fn is_dummy_string(val: &str) -> bool {
    let clean = val.trim();
    clean.eq_ignore_ascii_case("dammy")
        || clean.eq_ignore_ascii_case("<tbd>")
        || clean == "預備"
        || clean == "예비"
        || clean == "予備"
}

fn parse_inner(file_bytes: &[u8], separator: Option<Separator>) -> Result<StageName, StageNameError> {
    let file_content = file::scrub(file_bytes);
    let csv_separator = file::resolve(separator, &file_content);

    let mut parsed_lines: Vec<(usize, Vec<String>)> = Vec::new();

    for (line_index, file_line) in file_content.lines().enumerate() {
        let mut clean_line = file_line;
        if let Some((before_comment, _)) = file_line.split_once("//") {
            clean_line = before_comment;
        }

        let trimmed_line = clean_line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        let stage_names: Vec<String> = trimmed_line
            .split(csv_separator)
            .map(|part| part.trim().to_string())
            .collect();

        if stage_names.is_empty() {
            continue;
        }

        parsed_lines.push((line_index, stage_names));
    }

    if parsed_lines.is_empty() {
        return Err(StageNameError::EmptyFile);
    }

    let mut dummy_idx = None;
    for (idx, (_, names)) in parsed_lines.iter().enumerate() {
        if let Some(first_name) = names.first()
            && is_dummy_string(first_name) {
                dummy_idx = Some(idx);
                break;
            }
    }

    let mut stage_entries = HashMap::new();

    if let Some(d_idx) = dummy_idx {
        let valid_story_lines: Vec<Vec<String>> = parsed_lines
            .into_iter()
            .take(d_idx)
            .map(|(_, names)| names)
            .collect();

        let valid_lines_count = valid_story_lines.len();

        if valid_lines_count < 2 {
            for (idx, names) in valid_story_lines.into_iter().enumerate() {
                stage_entries.insert(idx as u32, StageNameEntry { names });
            }
        } else {
            let split_index = valid_lines_count - 2;
            let (backwards_part, forwards_part) = valid_story_lines.split_at(split_index);
            let mut assigned_stage_id = 0u32;

            for names in backwards_part.iter().rev() {
                stage_entries.insert(
                    assigned_stage_id,
                    StageNameEntry {
                        names: names.clone(),
                    },
                );
                assigned_stage_id += 1;
            }

            for names in forwards_part.iter() {
                stage_entries.insert(
                    assigned_stage_id,
                    StageNameEntry {
                        names: names.clone(),
                    },
                );
                assigned_stage_id += 1;
            }
        }
    } else {
        for (original_index, names) in parsed_lines {
            stage_entries.insert(original_index as u32, StageNameEntry { names });
        }
    }

    Ok(StageName {
        entries: stage_entries,
    })
}