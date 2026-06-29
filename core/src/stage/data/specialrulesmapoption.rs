use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, instrument};
use crate::global::resolver;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SpecialRuleOption {
    pub invalid_combo_ids: Vec<u32>,
}

#[derive(Deserialize)]
struct RawRuleOption {
    #[serde(rename = "InvalidNyancomboID", default)]
    invalid_combo_ids: Vec<u32>,
}

#[instrument(skip(dir, priority))]
pub fn load(dir: &Path, filename: &str, priority: &[String]) -> HashMap<u8, SpecialRuleOption> {
    let mut options_map = HashMap::new();
    let file_paths = resolver::get(dir, [filename], priority);

    let Some(target_path) = file_paths.first() else {
        debug!("Special rules option file not found");
        return options_map;
    };

    let Ok(file_content) = fs::read_to_string(target_path) else {
        error!(path = ?target_path, "Failed to read special rules option file");
        return options_map;
    };

    let Ok(json_data) = serde_json::from_str::<HashMap<String, RawRuleOption>>(&file_content) else {
        error!("Failed to deserialize special rules option JSON");
        return options_map;
    };

    for (rule_id_str, raw_option) in json_data {
        let Ok(rule_id) = rule_id_str.parse::<u8>() else { continue; };

        options_map.insert(rule_id, SpecialRuleOption {
            invalid_combo_ids: raw_option.invalid_combo_ids,
        });
    }

    options_map
}