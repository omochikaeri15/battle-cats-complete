use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::scrub;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SpecialRulesMapOptionError {
    InvalidJson,
}

impl fmt::Display for SpecialRulesMapOptionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson => write!(
                formatter,
                "The provided byte slice could not be parsed as valid JSON."
            ),
        }
    }
}

impl std::error::Error for SpecialRulesMapOptionError {}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecialRulesMapOptionEntry {
    pub invalid_combo_ids: Vec<u32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecialRulesMapOption {
    pub entries: HashMap<u8, SpecialRulesMapOptionEntry>,
}

impl SpecialRulesMapOption {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, SpecialRulesMapOptionError> {
        let clean_json = scrub(bytes.as_ref());
        parse_inner(&clean_json)
    }
}

fn parse_inner(json_str: &str) -> Result<SpecialRulesMapOption, SpecialRulesMapOptionError> {
    let json_value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|_| SpecialRulesMapOptionError::InvalidJson)?;

    let mut entries = HashMap::new();

    if let Some(rule_type_object) = json_value.get("RuleType").and_then(|v| v.as_object()) {
        for (rule_id_str, option_data) in rule_type_object {
            let Ok(rule_id) = rule_id_str.parse::<u8>() else { continue; };

            let invalid_combo_ids: Vec<u32> = option_data
                .get("InvalidNyancomboID")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_u64().map(|n| n as u32))
                        .collect()
                })
                .unwrap_or_default();

            entries.insert(
                rule_id,
                SpecialRulesMapOptionEntry {
                    invalid_combo_ids,
                },
            );
        }
    }

    Ok(SpecialRulesMapOption { entries })
}