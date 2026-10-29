use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::scrub;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScoreBonusMapError {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BonusType {
    Weaken(Vec<u32>),
    Freeze(Vec<u32>),
    Slow(Vec<u32>),
    Knockback(Vec<u32>),
    StrongAttack(Vec<u32>),
    MassiveDamage(Vec<u32>),
    StrongDefense(Vec<u32>),
    Resist(Vec<u32>),
    Unknown(u8, Vec<u32>),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScoreBonusMapEntry {
    pub bonuses: Vec<BonusType>,
    pub name_label: String,
    pub explanation_label: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScoreBonusMap {
    pub entries: HashMap<u32, ScoreBonusMapEntry>,
}

impl ScoreBonusMap {
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