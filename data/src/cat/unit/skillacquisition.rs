use std::collections::HashMap;
use std::fmt;

use crate::common::file::{self, Separator};

/// Represents errors that can occur during the parsing of talent configurations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SkillAcquisitionError {
    /// The supplied bytes yielded no parseable rows.
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

/// A unit's complete talent configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Talent {
    /// The identifier of the unit this configuration belongs to.
    pub id: u32,
    /// The structural classification flag for the talent layout.
    pub type_id: u16,
    /// The collection of individual talents.
    pub groups: Vec<TalentGroup>,
}

/// One upgradeable talent within a configuration.
///
/// Carries the boundary parameters and level cap needed to compute the talent's
/// effect at any level, plus the indices linking it to its cost curve and text.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TalentGroup {
    /// The internal ID specifying the mechanical effect of the talent.
    pub ability_id: u8,
    /// The maximum permitted level for this talent.
    pub max_level: u8,
    /// The base boundary parameter for the first data field.
    pub min_1: u16,
    /// The maximum boundary parameter for the first data field.
    pub max_1: u16,
    /// The base boundary parameter for the second data field.
    pub min_2: u16,
    /// The maximum boundary parameter for the second data field.
    pub max_2: u16,
    /// The base boundary parameter for the third data field.
    pub min_3: u16,
    /// The maximum boundary parameter for the third data field.
    pub max_3: u16,
    /// The base boundary parameter for the fourth data field.
    pub min_4: u16,
    /// The maximum boundary parameter for the fourth data field.
    pub max_4: u16,
    /// The index mapping to the associated localized explanation text.
    pub text_id: u8,
    /// The index mapping to the associated resource cost curve.
    pub cost_id: u8,
    /// The index mapping to the associated localized display name.
    pub name_id: i16,
    /// The maximum allowable instantiation limit for this specific talent.
    pub limit: u8,
}

impl TalentGroup {
    /// Interpolates a talent's effect value at a given progression level.
    ///
    /// The value scales linearly between the declared minimum and maximum
    /// boundaries across the talent's level range. Level zero yields no effect,
    /// the first level yields exactly the minimum, and the maximum level yields
    /// exactly the maximum, so the endpoints are never subject to rounding.
    ///
    /// # Arguments
    /// * `min` - The boundary parameter representing the effect at the first level.
    /// * `max` - The boundary parameter representing the effect at the maximum level.
    /// * `level` - The current progression level to evaluate.
    /// * `max_level` - The highest level this talent may reach.
    ///
    /// # Returns
    /// An `i32` containing the rounded effect value at the requested level, or
    /// zero when the talent is unlearned.
    pub fn calculate_value(min: u16, max: u16, level: u8, max_level: u8) -> i32 {
        if level == 0 { return 0; }
        if max_level <= 1 { return min as i32; }
        if level == 1 { return min as i32; }
        if level == max_level { return max as i32; }

        let minimum_float = min as f32;
        let maximum_float = max as f32;
        let level_float = level as f32;
        let maximum_level_float = max_level as f32;

        let calculated_value = minimum_float + (maximum_float - minimum_float) * (level_float - 1.0) / (maximum_level_float - 1.0);
        calculated_value.round() as i32
    }
}

impl Talent {
    /// Parses the talent acquisition table into configurations keyed by unit identifier.
    ///
    /// Each row declares one unit's talent layout followed by a variable-length
    /// run of fourteen-column talent groups, terminated by a group whose ability
    /// identifier is zero. Rows whose leading column is not a valid identifier
    /// are skipped, which discards any header the file carries.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the `SkillAcquisition.csv` file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed configurations keyed by unit identifier
    /// on success, or a `SkillAcquisitionError` if the file contained no
    /// parseable rows.
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
