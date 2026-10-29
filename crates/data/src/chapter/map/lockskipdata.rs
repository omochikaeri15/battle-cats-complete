//! Rules barring the stage skip feature from particular maps.
use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

/// Represents errors that can occur during the parsing of stage skip exclusions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LockSkipDataError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
}

impl fmt::Display for LockSkipDataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid lock skip data."
            ),
        }
    }
}

impl std::error::Error for LockSkipDataError {}

/// A single rule barring the stage skip feature from a map.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockSkipDataEntry {
    /// The identifier of the message shown to explain why skipping is barred.
    pub exclusion_message_type: u32,
    /// The identifier of the map the exclusion applies to.
    pub excluded_map_id: u32,
    /// The trailing comment text accompanying the row in the source file.
    pub comment: String,
}

/// The parsed contents of the stage skip exclusion table.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockSkipData {
    /// The exclusion rules, keyed by the excluded map identifier.
    pub entries: HashMap<u32, LockSkipDataEntry>,
}

impl LockSkipData {
    /// Parses the stage skip exclusion table into per-map rules.
    ///
    /// Trailing comment text introduced by a double slash is retained on the
    /// resulting entry rather than discarded, because the source file uses it to
    /// record why each exclusion exists.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the lock skip data file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed `LockSkipData` on success, or a
    /// `LockSkipDataError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, LockSkipDataError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<LockSkipData, LockSkipDataError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::resolve(separator, &file_content);

    let mut entries = HashMap::new();
    let mut has_content = false;

    for file_line in file_content.lines() {
        let mut data_part = file_line;
        let mut comment_part = "";

        if let Some((before_comment, after_comment)) = file_line.split_once("//") {
            data_part = before_comment;
            comment_part = after_comment;
        }

        let trimmed_data = data_part.trim();
        if trimmed_data.is_empty() {
            continue;
        }

        has_content = true;

        let parts: Vec<&str> = trimmed_data.split(separator_char).collect();

        let mut message_type = 0;
        if let Some(message_type_str) = parts.first()
            && let Ok(parsed_type) = message_type_str.trim().parse::<u32>() {
                message_type = parsed_type;
            }

        let Some(stage_id_str) = parts.get(1) else { continue; };
        let Ok(stage_id) = stage_id_str.trim().parse::<u32>() else { continue; };

        entries.insert(
            stage_id,
            LockSkipDataEntry {
                exclusion_message_type: message_type,
                excluded_map_id: stage_id,
                comment: comment_part.trim().to_string(),
            },
        );
    }

    if !has_content {
        return Err(LockSkipDataError::EmptyFile);
    }

    Ok(LockSkipData { entries })
}