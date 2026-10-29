use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FixedFormationError {
    EmptyFile,
    MissingHeaders,
}

impl fmt::Display for FixedFormationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid fixed formation data."
            ),
            Self::MissingHeaders => write!(
                formatter,
                "The file is missing the required header row."
            ),
        }
    }
}

impl std::error::Error for FixedFormationError {}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FixedFormationEntry {
    pub map_id: u32,
    pub level: u8,
    pub stage_no: u32,
    pub preset_file_name: String,
    pub memo: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FixedFormation {
    pub formations: HashMap<(u32, u8, u32), FixedFormationEntry>,
}

impl FixedFormation {
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, FixedFormationError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<FixedFormation, FixedFormationError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::resolve(separator, &file_content);

    let mut lines_iterator = file_content.lines();

    let Some(header_line) = lines_iterator.next() else {
        return Err(FixedFormationError::MissingHeaders);
    };

    let headers_map: HashMap<&str, usize> = header_line
        .split(separator_char)
        .enumerate()
        .map(|(index, header_string)| (header_string.trim(), index))
        .collect();

    let mut formations = HashMap::new();
    let mut has_content = false;

    for file_line in lines_iterator {
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

        let get_value = |header_name: &str, fallback_index: usize| -> Option<&str> {
            if let Some(&column_index) = headers_map.get(header_name)
                && let Some(value) = parts.get(column_index) {
                    return Some(value.trim());
                }
            if let Some(value) = parts.get(fallback_index) {
                return Some(value.trim());
            }
            None
        };

        let Some(map_id_string) = get_value("MapID", 0) else { continue; };
        let Ok(map_id) = map_id_string.parse::<u32>() else { continue; };

        let Some(level_string) = get_value("Level", 1) else { continue; };
        let Ok(level) = level_string.parse::<u8>() else { continue; };

        let Some(stage_no_string) = get_value("StageNo", 2) else { continue; };
        let Ok(stage_no) = stage_no_string.parse::<u32>() else { continue; };

        let Some(preset_file_name_string) = get_value("PresetFileName", 3) else { continue; };

        let memo_string = match get_value("MEMO", 4) {
            Some(found_memo) => found_memo.to_string(),
            None => String::new(),
        };

        formations.insert(
            (map_id, level, stage_no),
            FixedFormationEntry {
                map_id,
                level,
                stage_no,
                preset_file_name: preset_file_name_string.to_string(),
                memo: memo_string,
            },
        );
    }

    if !has_content {
        return Err(FixedFormationError::EmptyFile);
    }

    Ok(FixedFormation { formations })
}