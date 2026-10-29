//! Score bonuses awarded for fielding units with particular abilities.
use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::scrub;

/// Represents errors that can occur during the parsing of score bonus rules.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScoreBonusMapError {
    /// The supplied bytes were not valid JSON.
    InvalidJson,
}

impl fmt::Display for ScoreBonusMapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson => write!(
                formatter,
                "The provided byte slice could not be parsed as valid JSON."
            ),
        }
    }
}

impl std::error::Error for ScoreBonusMapError {}

/// A score bonus awarded for fielding units with a given ability.
///
/// Each variant carries the ability's qualifying parameters as declared in the
/// source document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BonusType {
    /// A bonus for units able to weaken opponents.
    Weaken(Vec<u32>),
    /// A bonus for units able to freeze opponents.
    Freeze(Vec<u32>),
    /// A bonus for units able to slow opponents.
    Slow(Vec<u32>),
    /// A bonus for units able to repel opponents.
    Knockback(Vec<u32>),
    /// A bonus for units carrying the Strong Against ability.
    StrongAttack(Vec<u32>),
    /// A bonus for units carrying the Massive Damage ability.
    MassiveDamage(Vec<u32>),
    /// A bonus for units carrying a defensive damage reduction ability.
    StrongDefense(Vec<u32>),
    /// A bonus for units carrying the Resistant ability.
    Resist(Vec<u32>),
    /// A bonus code this parser does not recognize, carrying its raw code and parameters.
    Unknown(u8, Vec<u32>),
}

/// The score bonus configuration for a single map.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScoreBonusMapEntry {
    /// The bonuses this map awards.
    pub bonuses: Vec<BonusType>,
    /// The localization key for the bonus set's display name.
    pub name_label: String,
    /// The localization key for the bonus set's explanatory text.
    pub explanation_label: String,
}

/// The parsed contents of the score bonus table.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScoreBonusMap {
    /// The bonus configurations, keyed by map identifier.
    pub entries: HashMap<u32, ScoreBonusMapEntry>,
}

impl ScoreBonusMap {
    /// Parses the score bonus document into per-map bonus configurations.
    ///
    /// Unlike most engine tables this source is JSON rather than delimited text,
    /// so the bytes are sanitized into UTF-8 before being decoded. Bonus codes
    /// the parser does not recognize are retained as `BonusType::Unknown` rather
    /// than discarded.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the score bonus JSON document.
    ///
    /// # Returns
    /// A `Result` containing the parsed `ScoreBonusMap` on success, or a
    /// `ScoreBonusMapError` if the bytes were not valid JSON.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, ScoreBonusMapError> {
        let clean_json = scrub(bytes.as_ref());
        parse_inner(&clean_json)
    }
}

fn parse_inner(json_str: &str) -> Result<ScoreBonusMap, ScoreBonusMapError> {
    let json_value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|_| ScoreBonusMapError::InvalidJson)?;

    let mut entries = HashMap::new();

    let Some(map_id_object) = json_value.get("MapID").and_then(|v| v.as_object()) else {
        return Ok(ScoreBonusMap { entries });
    };

    for (map_id_str, map_data) in map_id_object {
        let Ok(map_id) = map_id_str.parse::<u32>() else {
            continue;
        };

        let name_label = map_data
            .get("BonusNameLabel")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let explanation_label = map_data
            .get("BonusExplanationLabel")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let mut bonuses = Vec::new();

        if let Some(bonus_type_object) = map_data.get("BonusType").and_then(|v| v.as_object()) {
            for (key_str, type_data) in bonus_type_object {
                let Ok(key) = key_str.parse::<u8>() else {
                    continue;
                };

                let effect_values: Vec<u32> = type_data
                    .get("Parameters")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_u64().map(|n| n as u32))
                            .collect()
                    })
                    .unwrap_or_default();

                let bonus_enum = match key {
                    0 => BonusType::Weaken(effect_values),
                    1 => BonusType::Freeze(effect_values),
                    2 => BonusType::Slow(effect_values),
                    3 => BonusType::Knockback(effect_values),
                    13 => BonusType::StrongAttack(effect_values),
                    14 => BonusType::MassiveDamage(effect_values),
                    16 => BonusType::StrongDefense(effect_values),
                    17 => BonusType::Resist(effect_values),
                    _ => BonusType::Unknown(key, effect_values),
                };

                bonuses.push(bonus_enum);
            }
        }

        entries.insert(
            map_id,
            ScoreBonusMapEntry {
                bonuses,
                name_label,
                explanation_label,
            },
        );
    }

    Ok(ScoreBonusMap { entries })
}