//! Constraints that maps impose on how the player may fight them.
use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::scrub;

/// Represents errors that can occur during the parsing of map special rules.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SpecialRulesMapError {
    /// The supplied bytes were not valid JSON.
    InvalidJson,
}

impl fmt::Display for SpecialRulesMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The provided byte slice could not be parsed as valid JSON.")
    }
}

impl std::error::Error for SpecialRulesMapError {}

/// A constraint a map imposes on how the player may fight it.
///
/// Each variant carries the rule's parameters as declared in the source
/// document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleType {
    /// The player begins the stage with a fixed starting budget.
    TrustFund(Vec<u32>),
    /// Every unit is forced to share the same redeployment delay.
    CooldownEquality(Vec<u32>),
    /// The lineup may hold only so many units of each rarity.
    LineupLimit(Vec<u32>),
    /// Only so many units of each rarity may be on the field at once.
    RarityLimit(Vec<u32>),
    /// Every unit is deployed for the same fixed cost.
    CheapLabor(Vec<u32>),
    /// Unit deployment costs are scaled by a percentage per rarity.
    CatCost(Vec<u32>),
    /// The rate at which the budget accumulates is altered.
    CatProduction(Vec<u32>),
    /// The total number of units deployable over the stage is capped.
    TotalDeployLimit(Vec<u32>),
    /// Each unit must be deployed more than once to take effect.
    MoreThanOne(Vec<u32>),
    /// The base cannon is replaced with its mega variant.
    MegaCatCannon(Vec<u32>),
    /// Every unit is forced to share the same movement speed.
    UniformMotion(Vec<u32>),
    /// Each deployment raises the cost of the next.
    CompoundingCost(Vec<u32>),
    /// A rule code this parser does not recognize, carrying its raw code and parameters.
    Unknown(u8, Vec<u32>),
}

/// The special rule configuration for a single map.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecialRulesMapEntry {
    /// The identifier of the rule set's presentation category.
    pub contents_type: u8,
    /// The constraints this map imposes.
    pub rules: Vec<RuleType>,
    /// The localization key for the rule set's display name.
    pub name_label: String,
    /// The localization key for the rule set's explanatory text.
    pub explanation_label: String,
}

/// The parsed contents of the map special rule table.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecialRulesMap {
    /// The rule configurations, keyed by map identifier.
    pub entries: HashMap<u32, SpecialRulesMapEntry>,
}

impl SpecialRulesMap {
    /// Parses the map special rule document into per-map constraints.
    ///
    /// Unlike most engine tables this source is JSON rather than delimited text,
    /// so the bytes are sanitized into UTF-8 before being decoded. Rule codes the
    /// parser does not recognize are retained as `RuleType::Unknown` rather than
    /// discarded.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the special rules JSON document.
    ///
    /// # Returns
    /// A `Result` containing the parsed `SpecialRulesMap` on success, or a
    /// `SpecialRulesMapError` if the bytes were not valid JSON.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, SpecialRulesMapError> {
        let clean = scrub(bytes.as_ref());
        parse_inner(&clean)
    }
}

fn parse_inner(json_str: &str) -> Result<SpecialRulesMap, SpecialRulesMapError> {
    let val: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|_| SpecialRulesMapError::InvalidJson)?;

    let mut entries = HashMap::new();

    let Some(map_obj) = val.get("MapID").and_then(|v| v.as_object()) else {
        return Ok(SpecialRulesMap { entries });
    };

    for (id_str, data) in map_obj {
        let Ok(id) = id_str.parse::<u32>() else { continue; };

        let contents_type = data.get("ContentsType").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
        let name_label = data.get("RuleNameLabel").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let explanation_label = data.get("RuleExplanationLabel").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let mut rules = Vec::new();
        if let Some(rule_obj) = data.get("RuleType").and_then(|v| v.as_object()) {
            for (r_id_str, r_data) in rule_obj {
                let Ok(r_id) = r_id_str.parse::<u8>() else { continue; };

                let params: Vec<u32> = r_data
                    .get("Parameters")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|x| x.as_u64().map(|n| n as u32)).collect())
                    .unwrap_or_default();

                let rule = match r_id {
                    0 => RuleType::TrustFund(params),
                    1 => RuleType::CooldownEquality(params),
                    2 => RuleType::LineupLimit(params),
                    3 => RuleType::RarityLimit(params),
                    4 => RuleType::CheapLabor(params),
                    5 => RuleType::CatCost(params),
                    6 => RuleType::CatProduction(params),
                    7 => RuleType::TotalDeployLimit(params),
                    8 => RuleType::MoreThanOne(params),
                    9 => RuleType::MegaCatCannon(params),
                    10 => RuleType::UniformMotion(params),
                    11 => RuleType::CompoundingCost(params),
                    _ => RuleType::Unknown(r_id, params),
                };

                rules.push(rule);
            }
        }

        entries.insert(id, SpecialRulesMapEntry {
            contents_type,
            rules,
            name_label,
            explanation_label,
        });
    }

    Ok(SpecialRulesMap { entries })
}