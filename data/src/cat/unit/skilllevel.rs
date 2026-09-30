use std::collections::HashMap;
use std::fmt;

use crate::common::file::{self, Separator};

/// Represents errors that can occur during the parsing of talent cost curves.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SkillLevelError {
    /// The supplied bytes yielded no parseable rows.
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

/// The cost of each successive level of a talent.
///
/// One curve per cost identifier; each element is the cost of advancing to the
/// next level.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TalentCost {
    /// The sequence of resource costs per level progression.
    pub costs: Vec<u16>,
}

impl TalentCost {
    /// Parses the talent cost table into curves keyed by cost identifier.
    ///
    /// Each row is addressed by its leading column rather than by its position,
    /// because talent groups reference these curves through the cost identifier
    /// recorded on the group. Rows whose leading column is not a valid
    /// identifier are skipped, which discards any header the file carries.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the `SkillLevel.csv` file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed curves keyed by cost identifier on
    /// success, or a `SkillLevelError` if the file contained no parseable rows.
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