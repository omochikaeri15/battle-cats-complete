use std::collections::HashMap;
use std::fmt;

use crate::common::file::{self, Separator};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SkillLevelError {
    EmptyFile,
}

impl fmt::Display for SkillLevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid skill level data."),
        }
    }
}

impl std::error::Error for SkillLevelError {}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TalentCost {
    pub costs: Vec<u16>,
}

impl TalentCost {
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<HashMap<u8, Self>, SkillLevelError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<HashMap<u8, TalentCost>, SkillLevelError> {
    let file_content = file::scrub(bytes);
    let delimiter = file::resolve(separator, &file_content);

    let mut map = HashMap::new();

    for line in file_content.lines() {
        if line.trim().is_empty() { continue; }

        let parts: Vec<&str> = line.split(delimiter).collect();
        if parts.is_empty() { continue; }

        if let Ok(id) = parts[0].trim().parse::<u8>() {
            let costs: Vec<u16> = parts.iter()
                .skip(1)
                .filter_map(|s| s.trim().parse::<u16>().ok())
                .collect();
            map.insert(id, TalentCost { costs });
        }
    }

    if map.is_empty() {
        return Err(SkillLevelError::EmptyFile);
    }

    Ok(map)
}