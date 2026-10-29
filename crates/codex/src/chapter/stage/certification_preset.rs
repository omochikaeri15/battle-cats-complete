use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::common::file::scrub;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CertificationPresetError {
    InvalidJson,
}

impl fmt::Display for CertificationPresetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson => write!(
                formatter,
                "The provided byte slice could not be parsed as valid JSON."
            ),
        }
    }
}

impl std::error::Error for CertificationPresetError {}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CannonType {
    #[default]
    Basic,
    SlowBeam,
    IronWall,
    Thunderbolt,
    Waterblast,
    HolyBlast,
    Breakerblast,
    Curseblast,
    Unknown(u8),
}

impl From<u8> for CannonType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Basic,
            1 => Self::SlowBeam,
            2 => Self::IronWall,
            3 => Self::Thunderbolt,
            4 => Self::Waterblast,
            5 => Self::HolyBlast,
            6 => Self::Breakerblast,
            7 => Self::Curseblast,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AbilityType {
    CatCannonAttack,
    CatCannonRange,
    CatCannonCharge,
    WorkerCatRate,
    WorkerCatWallet,
    BaseDefense,
    Research,
    BountyUp,
    Study,
    CatEnergy,
    Unknown(u8),
}

impl From<u8> for AbilityType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::CatCannonAttack,
            1 => Self::CatCannonRange,
            2 => Self::CatCannonCharge,
            3 => Self::WorkerCatRate,
            4 => Self::WorkerCatWallet,
            5 => Self::BaseDefense,
            6 => Self::Research,
            7 => Self::BountyUp,
            8 => Self::Study,
            9 => Self::CatEnergy,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TreasureType {
    EoC1,
    EoC2,
    EoC3,
    ItF1,
    ItF2,
    ItF3,
    CotC1,
    CotC2,
    CotC3,
    Unknown(u8),
}

impl From<u8> for TreasureType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::EoC1,
            1 => Self::EoC2,
            2 => Self::EoC3,
            4 => Self::ItF1,
            5 => Self::ItF2,
            6 => Self::ItF3,
            7 => Self::CotC1,
            8 => Self::CotC2,
            9 => Self::CotC3,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvolutionForm {
    #[default]
    Normal,
    Evolved,
    True,
    Ultra,
    Unknown(u8),
}

impl From<u8> for EvolutionForm {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Normal,
            2 => Self::Evolved,
            3 => Self::True,
            4 => Self::Ultra,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresetChara {
    pub evolution_form: EvolutionForm,
    pub level: u16,
    pub plus_level: u16,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresetAbility {
    pub level: u16,
    pub plus_level: u16,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresetTreasure {
    pub inferior_count: u8,
    pub normal_count: u8,
    pub superior_count: u8,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CertificationPreset {
    pub characters: HashMap<u32, PresetChara>,
    pub slot_units: Vec<u32>,
    pub slot_cannon_type: CannonType,
    pub abilities: HashMap<AbilityType, PresetAbility>,
    pub cannon_levels: HashMap<CannonType, u16>,
    pub treasures: HashMap<TreasureType, PresetTreasure>,
}

impl CertificationPreset {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, CertificationPresetError> {
        let clean_json = scrub(bytes.as_ref());
        parse_inner(&clean_json)
    }
}

fn parse_inner(json_str: &str) -> Result<CertificationPreset, CertificationPresetError> {
    let Ok(json_root) = serde_json::from_str::<Value>(json_str) else {
        return Err(CertificationPresetError::InvalidJson);
    };

    let mut lineup = CertificationPreset::default();

    extract_characters(&json_root, &mut lineup);
    extract_slot(&json_root, &mut lineup);
    extract_abilities(&json_root, &mut lineup);
    extract_cannons(&json_root, &mut lineup);
    extract_treasures(&json_root, &mut lineup);

    Ok(lineup)
}

fn extract_characters(json_root: &Value, lineup: &mut CertificationPreset) {
    let Some(base_value) = json_root.get("chara") else { return; };
    let Some(data_value) = base_value.get("data") else { return; };
    let Some(chara_map) = data_value.as_object() else { return; };

    for (unit_id_str, chara_value) in chara_map {
        let is_removed = chara_value.get("remove").and_then(|remove_val| remove_val.as_bool()) == Some(true);
        if is_removed {
            continue;
        }

        let Ok(unit_id) = unit_id_str.parse::<u32>() else { continue; };

        let Some(evolution_value) = chara_value.get("evolution") else { continue; };
        let Some(evolution_str) = evolution_value.as_str() else { continue; };

        let Some(level_value) = chara_value.get("level") else { continue; };
        let Some(level_str) = level_value.as_str() else { continue; };

        let Some(plus_value) = chara_value.get("plus") else { continue; };
        let Some(plus_str) = plus_value.as_str() else { continue; };

        let Ok(raw_evolution_id) = evolution_str.parse::<u8>() else { continue; };
        let Ok(level) = level_str.parse::<u16>() else { continue; };
        let Ok(plus_level) = plus_str.parse::<u16>() else { continue; };

        let evolution_form = EvolutionForm::from(raw_evolution_id);

        lineup.characters.insert(
            unit_id,
            PresetChara {
                evolution_form,
                level,
                plus_level,
            },
        );
    }
}

fn extract_slot(json_root: &Value, lineup: &mut CertificationPreset) {
    let Some(base_value) = json_root.get("slot") else { return; };
    let Some(data_value) = base_value.get("data") else { return; };
    let Some(slot_zero) = data_value.get("0") else { return; };

    if let Some(cannon_value) = slot_zero.get("cannon") {
        let parsed_cannon_id = match cannon_value {
            Value::Number(number_val) => number_val.as_u64().map(|parsed_val| parsed_val as u8),
            Value::String(string_val) => string_val.parse::<u8>().ok(),
            _ => None,
        };

        if let Some(cannon_id) = parsed_cannon_id {
            lineup.slot_cannon_type = CannonType::from(cannon_id);
        }
    }

    let Some(characters_value) = slot_zero.get("chara") else { return; };
    let Some(char_array) = characters_value.as_array() else { return; };

    for char_value in char_array {
        let parsed_char_id = match char_value {
            Value::Number(number_val) => number_val.as_u64().map(|parsed_val| parsed_val as u32),
            Value::String(string_val) => string_val.parse::<u32>().ok(),
            _ => None,
        };

        if let Some(character_id) = parsed_char_id {
            lineup.slot_units.push(character_id);
        }
    }
}

fn extract_abilities(json_root: &Value, lineup: &mut CertificationPreset) {
    let Some(base_value) = json_root.get("ability") else { return; };
    let Some(data_value) = base_value.get("data") else { return; };
    let Some(ability_map) = data_value.as_object() else { return; };

    for (ability_id_str, ability_value) in ability_map {
        let Ok(ability_id) = ability_id_str.parse::<u8>() else { continue; };
        let ability_type = AbilityType::from(ability_id);

        let mut level = 0;
        if let Some(level_value) = ability_value.get("level")
            && let Some(string_val) = level_value.as_str()
                && let Ok(parsed_level) = string_val.parse::<u16>() {
                    level = parsed_level;
                }

        let mut plus_level = 0;
        if let Some(plus_value) = ability_value.get("plus")
            && let Some(string_val) = plus_value.as_str()
                && let Ok(parsed_plus) = string_val.parse::<u16>() {
                    plus_level = parsed_plus;
                }

        lineup.abilities.insert(
            ability_type,
            PresetAbility {
                level,
                plus_level,
            },
        );
    }
}

fn extract_cannons(json_root: &Value, lineup: &mut CertificationPreset) {
    let Some(base_value) = json_root.get("cannon") else { return; };
    let Some(data_value) = base_value.get("data") else { return; };
    let Some(cannon_map) = data_value.as_object() else { return; };

    for (cannon_id_str, cannon_value) in cannon_map {
        let Ok(cannon_id) = cannon_id_str.parse::<u8>() else { continue; };
        let cannon_type = CannonType::from(cannon_id);

        let Some(level_value) = cannon_value.get("level") else { continue; };
        let Some(level_str) = level_value.as_str() else { continue; };
        let Ok(level) = level_str.parse::<u16>() else { continue; };

        lineup.cannon_levels.insert(cannon_type, level);
    }
}

fn extract_treasures(json_root: &Value, lineup: &mut CertificationPreset) {
    let Some(base_value) = json_root.get("treasure") else { return; };
    let Some(data_value) = base_value.get("data") else { return; };
    let Some(treasure_map) = data_value.as_object() else { return; };

    for (treasure_id_str, treasure_value) in treasure_map {
        let Ok(treasure_id) = treasure_id_str.parse::<u8>() else { continue; };
        let treasure_type = TreasureType::from(treasure_id);

        let Some(count_value) = treasure_value.get("count") else { continue; };
        let Some(counts_array) = count_value.as_array() else { continue; };

        let Some(inferior_value) = counts_array.first() else { continue; };
        let Some(inferior_str) = inferior_value.as_str() else { continue; };
        let Ok(inferior_count) = inferior_str.parse::<u8>() else { continue; };

        let Some(normal_value) = counts_array.get(1) else { continue; };
        let Some(normal_str) = normal_value.as_str() else { continue; };
        let Ok(normal_count) = normal_str.parse::<u8>() else { continue; };

        let Some(superior_value) = counts_array.get(2) else { continue; };
        let Some(superior_str) = superior_value.as_str() else { continue; };
        let Ok(superior_count) = superior_str.parse::<u8>() else { continue; };

        lineup.treasures.insert(
            treasure_type,
            PresetTreasure {
                inferior_count,
                normal_count,
                superior_count,
            },
        );
    }
}