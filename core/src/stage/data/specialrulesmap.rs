use std::fs;
use std::path::Path;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, instrument};
use crate::global::resolver;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RuleType {
    TrustFund(Vec<u32>),
    CooldownEquality(Vec<u32>),
    RarityLimit(Vec<u32>),
    CheapLabor(Vec<u32>),
    RestrictPrice(Vec<u32>),
    RestrictCd(Vec<u32>),
    DeployLimit(Vec<u32>),
    AwesomeCatSpawn(Vec<u32>),
    AwesomeCatCannon(Vec<u32>),
    AwesomeUnitSpeed(Vec<u32>),
    Unknown(u8, Vec<u32>),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SpecialRule {
    pub contents_type: u8,
    pub rules: Vec<RuleType>,
    pub name_label: String,
}

#[derive(Deserialize)]
struct RawRuleType {
    #[serde(rename = "Parameters")]
    parameters: Vec<u32>,
}

#[derive(Deserialize)]
struct RawRuleData {
    #[serde(rename = "ContentsType")]
    contents_type: u8,
    #[serde(rename = "RuleType", default)]
    rule_type: HashMap<String, RawRuleType>,
    #[serde(rename = "RuleNameLabel")]
    rule_name_label: Option<String>,
}

#[derive(Deserialize)]
struct RulesMap {
    #[serde(rename = "MapID", default)]
    map_id: HashMap<String, RawRuleData>,
}

#[instrument(skip(dir, priority))]
pub fn load(dir: &Path, filename: &str, priority: &[String]) -> HashMap<u32, SpecialRule> {
    let mut map = HashMap::new();
    let file_paths = resolver::get(dir, [filename], priority);

    let Some(target_path) = file_paths.first() else {
        debug!("Special rules map file not found");
        return map;
    };

    let Ok(file_content) = fs::read_to_string(target_path) else {
        error!(path = ?target_path, "Failed to read special rules map file");
        return map;
    };

    let Ok(json_data) = serde_json::from_str::<RulesMap>(&file_content) else {
        error!("Failed to deserialize special rules map JSON");
        return map;
    };

    for (map_id_str, raw_data) in json_data.map_id {
        let Ok(map_id) = map_id_str.parse::<u32>() else { continue; };

        let mut rules = Vec::new();
        for (rule_id_str, raw_type) in raw_data.rule_type {
            let Ok(rule_id) = rule_id_str.parse::<u8>() else { continue; };
            let parameters = raw_type.parameters;

            let rule_enum = match rule_id {
                0 => RuleType::TrustFund(parameters),
                1 => RuleType::CooldownEquality(parameters),
                3 => RuleType::RarityLimit(parameters),
                4 => RuleType::CheapLabor(parameters),
                5 => RuleType::RestrictPrice(parameters),
                6 => RuleType::RestrictCd(parameters),
                7 => RuleType::DeployLimit(parameters),
                8 => RuleType::AwesomeCatSpawn(parameters),
                9 => RuleType::AwesomeCatCannon(parameters),
                10 => RuleType::AwesomeUnitSpeed(parameters),
                _ => RuleType::Unknown(rule_id, parameters),
            };

            rules.push(rule_enum);
        }

        map.insert(map_id, SpecialRule {
            contents_type: raw_data.contents_type,
            rules,
            name_label: raw_data.rule_name_label.unwrap_or_default(),
        });
    }

    map
}