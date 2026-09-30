//! Units unlocked as a reward for clearing particular stages.
use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

/// Represents errors that can occur during the parsing of unit unlock drops.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DropCharaError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
}

impl fmt::Display for DropCharaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid character drop data."
            ),
        }
    }
}

impl std::error::Error for DropCharaError {}

/// The parsed contents of the unit unlock drop table.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DropChara {
    /// The identifier of the unit each stage unlocks, keyed by stage identifier.
    pub character_drops: HashMap<u32, u32>,
}

impl DropChara {
    /// Parses the unit unlock drop table into a mapping from stage to unit.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the character drop file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed `DropChara` on success, or a
    /// `DropCharaError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, DropCharaError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<DropChara, DropCharaError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::resolve(separator, &file_content);

    let mut character_drops = HashMap::new();
    let mut has_content = false;

    for file_line in file_content.lines().skip(1) {
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
        if parts.len() < 3 {
            continue;
        }

        let Some(drop_id_string) = parts.first() else { continue; };
        let Ok(stage_drop_chara_id) = drop_id_string.trim().parse::<i32>() else { continue; };

        if stage_drop_chara_id < 0 {
            continue;
        }

        let Some(resolved_chara_string) = parts.get(2) else { continue; };
        let Ok(resolved_chara_id) = resolved_chara_string.trim().parse::<u32>() else { continue; };

        character_drops.insert(stage_drop_chara_id as u32, resolved_chara_id);
    }

    if !has_content {
        return Err(DropCharaError::EmptyFile);
    }

    Ok(DropChara { character_drops })
}