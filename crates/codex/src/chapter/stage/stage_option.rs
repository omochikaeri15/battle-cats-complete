use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::columns;
use crate::common::file::{self, Separator};
use crate::common::columns::Column;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StageOptionError {
    EmptyFile,
}

impl fmt::Display for StageOptionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid stage option data."
            ),
        }
    }
}

impl std::error::Error for StageOptionError {}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageOptionEntry {
    pub map_id: u32,
    pub target_crowns: i8,
    pub target_stage: i32,
    pub rarity_mask: u8,
    pub deploy_limit: u32,
    pub allowed_rows: u8,
    pub min_cost: u32,
    pub max_cost: u32,
    pub charagroup_id: u32,
}

impl StageOptionEntry {
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        map_id        : 0;
        target_crowns : 1, Raw, -1;
        target_stage  : 2, Raw, -1;
        rarity_mask   : 3;
        deploy_limit  : 4;
        allowed_rows  : 5;
        min_cost      : 6;
        max_cost      : 7;
        charagroup_id : 8;
    };
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageOption {
    pub entries: HashMap<u32, Vec<StageOptionEntry>>,
}

impl StageOption {
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, StageOptionError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<StageOption, StageOptionError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::resolve(separator, &file_content);

    let mut entries: HashMap<u32, Vec<StageOptionEntry>> = HashMap::new();
    let mut has_content = false;

    let lines_iterator = file_content.lines().skip(1);

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
        if parts.len() < 9 {
            continue;
        }

        let Some(raw_map_id_str) = parts.first() else { continue; };
        let Ok(map_id) = raw_map_id_str.trim().parse::<u32>() else { continue; };

        let mut entry = StageOptionEntry::default();
        columns::apply(&parts, StageOptionEntry::COLUMNS, &mut entry);

        entries.entry(map_id).or_default().push(entry);
    }

    if !has_content {
        return Err(StageOptionError::EmptyFile);
    }

    Ok(StageOption { entries })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_column_reaches_a_field_of_its_own() {
        columns::assert_one_field_per_column(StageOptionEntry::COLUMNS);
    }
}
