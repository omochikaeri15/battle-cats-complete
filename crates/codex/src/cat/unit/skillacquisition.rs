use std::collections::HashMap;
use std::fmt;

use crate::common::file::{self, Separator};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SkillAcquisitionError {
    EmptyFile,
}

impl fmt::Display for SkillAcquisitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid talent data."),
        }
    }
}

impl std::error::Error for SkillAcquisitionError {}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Talent {
    pub id: u32,
    pub type_id: u16,
    pub groups: Vec<TalentGroup>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TalentGroup {
    pub ability_id: u8,
    pub max_level: u8,
    pub min_1: u16,
    pub max_1: u16,
    pub min_2: u16,
    pub max_2: u16,
    pub min_3: u16,
    pub max_3: u16,
    pub min_4: u16,
    pub max_4: u16,
    pub text_id: u8,
    pub cost_id: u8,
    pub name_id: i16,
    pub limit: u8,
}

impl TalentGroup {
    pub fn calculate_value(min: u16, max: u16, level: u8, max_level: u8) -> i32 {
        if level == 0 { return 0; }
        if max_level <= 1 { return min as i32; }

        let span = max as i32 - min as i32;
        let steps = max_level as i32 - 1;

        min as i32 + (level as i32 - 1) * span / steps
    }
}

impl Talent {
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<HashMap<u32, Self>, SkillAcquisitionError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<HashMap<u32, Talent>, SkillAcquisitionError> {
    let file_content = file::scrub(bytes);
    let delimiter = file::resolve(separator, &file_content);

    let mut map = HashMap::new();

    for line in file_content.lines() {
        let parts: Vec<&str> = line.split(delimiter).collect();
        if parts.len() < 2 { continue; }

        let Ok(id) = parts[0].trim().parse::<u32>() else { continue; };

        let type_id = parts[1].trim().parse::<u16>().unwrap_or(0);
        let mut groups = Vec::new();
        let mut index = 2;

        while index + 13 < parts.len() {
            let ability_id = parts[index].trim().parse::<u8>().unwrap_or(0);
            if ability_id == 0 { break; }

            let group = TalentGroup {
                ability_id,
                max_level: parts[index+1].trim().parse().unwrap_or(0),
                min_1: parts[index+2].trim().parse().unwrap_or(0), max_1: parts[index+3].trim().parse().unwrap_or(0),
                min_2: parts[index+4].trim().parse().unwrap_or(0), max_2: parts[index+5].trim().parse().unwrap_or(0),
                min_3: parts[index+6].trim().parse().unwrap_or(0), max_3: parts[index+7].trim().parse().unwrap_or(0),
                min_4: parts[index+8].trim().parse().unwrap_or(0), max_4: parts[index+9].trim().parse().unwrap_or(0),
                text_id: parts[index+10].trim().parse().unwrap_or(0),
                cost_id: parts[index+11].trim().parse().unwrap_or(0),
                name_id: parts[index+12].trim().parse().unwrap_or(-1),
                limit: parts[index+13].trim().parse().unwrap_or(0),
            };
            groups.push(group);
            index += 14;
        }

        map.insert(id, Talent { id, type_id, groups });
    }

    if map.is_empty() {
        return Err(SkillAcquisitionError::EmptyFile);
    }

    Ok(map)
}
