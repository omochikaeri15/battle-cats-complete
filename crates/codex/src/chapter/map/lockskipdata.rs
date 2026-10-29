use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LockSkipDataError {
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

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockSkipDataEntry {
    pub exclusion_message_type: u32,
    pub excluded_map_id: u32,
    pub comment: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockSkipData {
    pub entries: HashMap<u32, LockSkipDataEntry>,
}

impl LockSkipData {
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