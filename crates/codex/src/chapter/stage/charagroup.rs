use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CharaGroupError {
    EmptyFile,
}

impl fmt::Display for CharaGroupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid character group data."
            ),
        }
    }
}

impl std::error::Error for CharaGroupError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharaGroupType {
    OnlyUse,
    CannotUse,
    Unknown(u32),
}

impl From<u32> for CharaGroupType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::OnlyUse,
            2 => Self::CannotUse,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharaGroupEntry {
    pub id: u32,
    pub text_id: Option<String>,
    pub kind: CharaGroupType,
    pub units: Vec<u32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharaGroup {
    pub groups: HashMap<u32, CharaGroupEntry>,
}

impl CharaGroup {
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, CharaGroupError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<CharaGroup, CharaGroupError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::resolve(separator, &file_content);

    let mut groups = HashMap::new();
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

        let Some(id_str) = parts.first() else { continue; };
        let Ok(id) = id_str.trim().parse::<u32>() else { continue; };

        let text_id = parts
            .get(1)
            .map(|key| key.trim())
            .filter(|key| !key.is_empty())
            .map(str::to_owned);

        let Some(kind_str) = parts.get(2) else { continue; };
        let parsed_kind = kind_str.trim().parse::<u32>().unwrap_or(0);

        let mut units = Vec::new();
        for unit_str in parts.iter().skip(3) {
            if let Ok(unit_id) = unit_str.trim().parse::<u32>() {
                units.push(unit_id);
            }
        }

        groups.insert(
            id,
            CharaGroupEntry {
                id,
                text_id,
                kind: CharaGroupType::from(parsed_kind),
                units,
            },
        );
    }

    if !has_content {
        return Err(CharaGroupError::EmptyFile);
    }

    Ok(CharaGroup { groups })
}

#[cfg(test)]
mod tests {
    use super::*;

    const REAL_TABLE: &str = "//group_id,text_id,group_type,chara_id\n\
        1,stage_restriction_charagroup_1,0,249,250,251\n\
        10,Nyancombo_charagroup_1,3,831,832,833,834,835,836,857,858";

    fn parse_one(table: &str, id: u32) -> CharaGroupEntry {
        CharaGroup::parse(table, None).unwrap().groups.remove(&id).unwrap()
    }

    #[test]
    fn a_real_row_lands_column_for_column() {
        let stage = parse_one(REAL_TABLE, 1);

        assert_eq!(stage.id, 1);
        assert_eq!(stage.text_id.as_deref(), Some("stage_restriction_charagroup_1"));
        assert_eq!(stage.kind, CharaGroupType::OnlyUse);
        assert_eq!(stage.units, [249, 250, 251]);
    }

    #[test]
    fn a_combo_row_keeps_the_key_that_names_its_restriction() {
        let combo = parse_one(REAL_TABLE, 10);

        assert_eq!(combo.text_id.as_deref(), Some("Nyancombo_charagroup_1"));
        assert_eq!(combo.kind, CharaGroupType::Unknown(3));
        assert_eq!(combo.units, [831, 832, 833, 834, 835, 836, 857, 858]);
    }

    #[test]
    fn a_row_naming_no_restriction_reads_as_absent() {
        let entry = parse_one("//header\n4,,2,626", 4);

        assert_eq!(entry.text_id, None);
        assert_eq!(entry.kind, CharaGroupType::CannotUse);
    }

    #[test]
    fn a_file_without_rows_is_rejected() {
        assert_eq!(CharaGroup::parse("", None), Err(CharaGroupError::EmptyFile));
    }
}
