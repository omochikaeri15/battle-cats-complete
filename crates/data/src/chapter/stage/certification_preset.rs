//! Predetermined player states imposed on stages that equalize progression.

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::common::file::scrub;

/// Represents errors that can occur during the parsing of a fixed lineup preset.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CertificationPresetError {
    /// The supplied bytes were not valid JSON.
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

/// Identifies one of the base cannon's interchangeable effect modes.
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CannonType {
    /// The unmodified cannon, which deals damage and knocks back.
    #[default]
    Basic,
    /// The Slow Beam mode, which slows the enemies it strikes.
    SlowBeam,
    /// The Iron Wall mode, which raises a defensive barrier.
    IronWall,
    /// The Thunderbolt mode, which freezes the enemies it strikes.
    Thunderbolt,
    /// The Waterblast mode, which deals increased damage.
    Waterblast,
    /// The Holy Blast mode, which counters surges and waves.
    HolyBlast,
    /// The Breakerblast mode, which destroys barriers and shields.
    Breakerblast,
    /// The Curseblast mode, which curses the enemies it strikes.
    Curseblast,
    /// A cannon code this parser does not recognize, carrying its raw value.
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

/// Identifies one of the permanently upgradeable base abilities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AbilityType {
    /// The base cannon's damage output.
    CatCannonAttack,
    /// The base cannon's effective range.
    CatCannonRange,
    /// The base cannon's recharge rate.
    CatCannonCharge,
    /// The rate at which the worker cat accumulates budget.
    WorkerCatRate,
    /// The maximum budget the worker cat may hold.
    WorkerCatWallet,
    /// The player base's health pool.
    BaseDefense,
    /// The reduction applied to unit redeployment delays.
    Research,
    /// The increase applied to currency earned from defeats.
    BountyUp,
    /// The increase applied to experience earned from clears.
    Study,
    /// The player's maximum energy reserve.
    CatEnergy,
    /// An ability code this parser does not recognize, carrying its raw value.
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

/// Identifies the chapter a treasure set was collected from.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TreasureType {
    /// Treasures from the first Empire of Cats chapter.
    EoC1,
    /// Treasures from the second Empire of Cats chapter.
    EoC2,
    /// Treasures from the third Empire of Cats chapter.
    EoC3,
    /// Treasures from the first Into the Future chapter.
    ItF1,
    /// Treasures from the second Into the Future chapter.
    ItF2,
    /// Treasures from the third Into the Future chapter.
    ItF3,
    /// Treasures from the first Cats of the Cosmos chapter.
    CotC1,
    /// Treasures from the second Cats of the Cosmos chapter.
    CotC2,
    /// Treasures from the third Cats of the Cosmos chapter.
    CotC3,
    /// A treasure code this parser does not recognize, carrying its raw value.
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

/// Identifies which evolutionary form of a unit a preset fields.
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvolutionForm {
    /// The unit's base form.
    #[default]
    Normal,
    /// The unit's first evolution.
    Evolved,
    /// The unit's True form.
    True,
    /// The unit's Ultra form.
    Ultra,
    /// A form code this parser does not recognize, carrying its raw value.
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

/// The state a preset fields one unit at.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresetChara {
    /// The evolutionary form the unit is fielded in.
    pub evolution_form: EvolutionForm,
    /// The unit's ordinary level.
    pub level: u16,
    /// The unit's plus level, accumulated beyond the ordinary cap.
    pub plus_level: u16,
}

/// The state a preset grants one base ability at.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresetAbility {
    /// The ability's ordinary level.
    pub level: u16,
    /// The ability's plus level, accumulated beyond the ordinary cap.
    pub plus_level: u16,
}

/// The treasures a preset grants from one chapter.
///
/// The three grades stack, so each is counted separately.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresetTreasure {
    /// The number of lowest-grade treasures granted.
    pub inferior_count: u8,
    /// The number of middle-grade treasures granted.
    pub normal_count: u8,
    /// The number of highest-grade treasures granted.
    pub superior_count: u8,
}

/// A complete predetermined player state imposed on a stage.
///
/// A preset replaces the player's roster, upgrades, and treasures entirely, so
/// every attempt is fought under identical conditions.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CertificationPreset {
    /// The state each available unit is fielded at, keyed by unit identifier.
    pub characters: HashMap<u32, PresetChara>,
    /// The identifiers of the units occupying the lineup slots, in slot order.
    pub slot_units: Vec<u32>,
    /// The cannon mode the base is equipped with.
    pub slot_cannon_type: CannonType,
    /// The level each base ability is granted at, keyed by ability.
    pub abilities: HashMap<AbilityType, PresetAbility>,
    /// The level each cannon mode is granted at, keyed by mode.
    pub cannon_levels: HashMap<CannonType, u16>,
    /// The treasures granted from each chapter, keyed by chapter.
    pub treasures: HashMap<TreasureType, PresetTreasure>,
}

impl CertificationPreset {
    /// Parses a fixed lineup preset document into a complete player state.
    ///
    /// This source is JSON rather than delimited text. Unrecognized codes are
    /// retained in their respective `Unknown` variants rather than discarded.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the preset JSON document.
    ///
    /// # Returns
    /// A `Result` containing the parsed `CertificationPreset` on success, or a
    /// `CertificationPresetError` if the bytes were not valid JSON.
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