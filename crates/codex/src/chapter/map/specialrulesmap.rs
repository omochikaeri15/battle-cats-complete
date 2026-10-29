use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::scrub;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SpecialRulesMapError {
    InvalidJson,
}

impl fmt::Display for SpecialRulesMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The provided byte slice could not be parsed as valid JSON.")
    }
}

impl std::error::Error for SpecialRulesMapError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleType {
    TrustFund(Vec<u32>),
    CooldownEquality(Vec<u32>),
    LineupLimit(Vec<u32>),
    RarityLimit(Vec<u32>),
    CheapLabor(Vec<u32>),
    CatCost(Vec<u32>),
    CatProduction(Vec<u32>),
    TotalDeployLimit(Vec<u32>),
    MoreThanOne(Vec<u32>),
    MegaCatCannon(Vec<u32>),
    UniformMotion(Vec<u32>),
    CompoundingCost(Vec<u32>),
    Unknown(u8, Vec<u32>),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecialRulesMapEntry {
    pub contents_type: u8,
    pub rules: Vec<RuleType>,
    pub name_label: String,
    pub explanation_label: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecialRulesMap {
    pub entries: HashMap<u32, SpecialRulesMapEntry>,
}

impl SpecialRulesMap {
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