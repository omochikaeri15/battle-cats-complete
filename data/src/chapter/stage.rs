//! Public facade for stage-level data.
//!
//! This module re-exports the aggregate [`Stage`] structure alongside every
//! specialized parser and error type that contributes to it.

mod battleground;
mod certification_preset;
mod charagroup;
mod cost;
mod difficulty_level;
mod drop_chara;
mod fixed_formation;
mod mapstagedata;
mod scatcpusetting;
mod stage_option;
mod stagename;
mod xp;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::chapter::Category;

pub use battleground::{Battleground, BattlegroundEntry, BattlegroundError, BossType, EnemyAmount};
pub use certification_preset::{AbilityType, CannonType, CertificationPreset, CertificationPresetError, EvolutionForm, PresetAbility, PresetChara, PresetTreasure, TreasureType};
pub use charagroup::{CharaGroup, CharaGroupEntry, CharaGroupError, CharaGroupType};
pub use cost::{resolve_energy, CataminGrade, CostType, ResolvedCost};
pub use difficulty_level::{DifficultyLevel, DifficultyLevelError};
pub use drop_chara::{DropChara, DropCharaError};
pub use fixed_formation::{FixedFormation, FixedFormationEntry, FixedFormationError};
pub use mapstagedata::{DropReward, MapStageData, MapStageDataEntry, MapStageDataError, MapStageDataHeader, RewardStructure, TimedScore};
pub use scatcpusetting::{ScatCpuSetting, ScatCpuSettingError};
pub use stage_option::{StageOption, StageOptionEntry, StageOptionError};
pub use stagename::{StageName, StageNameEntry, StageNameError};
pub use xp::get_hardcoded_xp;

/// The fully-aggregated representation of a single stage.
///
/// The engine scatters a stage's definition across its own layout file plus the
/// shared name, metadata, difficulty, and restriction tables. This structure
/// combines them into one payload, leaving as `None` or empty the parts a given
/// stage does not participate in.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stage {
    /// The localized display name of the stage.
    pub name: String,
    /// The chapter grouping this stage belongs to.
    pub category: Category,
    /// The identifier of the map containing this stage.
    pub map_id: u32,
    /// The index of this stage within its map.
    pub stage_id: u32,
    /// The identifier of the enemy base defending the stage.
    pub base_id: i32,
    /// The identifier of the animation used for the enemy base.
    pub anim_base_id: u32,
    /// The horizontal extent of the battlefield in engine distance units.
    pub width: u32,
    /// The health pool of the enemy base.
    pub base_hp: u32,
    /// The shortest delay in frames between enemy spawn checks.
    pub min_spawn: u32,
    /// The longest delay in frames between enemy spawn checks.
    pub max_spawn: u32,
    /// The identifier of the background artwork the stage is drawn against.
    pub background_id: u32,
    /// The greatest number of enemies permitted on the field at once.
    pub max_enemies: u32,
    /// The stage's time limit in frames, or zero when it is untimed.
    pub time_limit: u32,
    /// Whether the player is barred from continuing after a loss.
    pub is_no_continues: bool,
    /// Whether the enemy base cannot be destroyed by ordinary means.
    pub is_base_indestructible: bool,
    /// A column present in the raw data whose effect on the engine is not established.
    pub unknown_value: u32,
    /// The enemy spawn configurations that make up the stage.
    pub enemies: Vec<BattlegroundEntry>,
    /// The cost of attempting the stage, in energy unless the stage's chapter or
    /// map metadata packs a currency into it.
    pub cost: u32,
    /// The cost scheme this stage's map declares, before any chapter override.
    pub declared_cost_type: CostType,
    /// The experience awarded for clearing the stage.
    pub xp: u32,
    /// The identifier of the music track played from the start of the stage.
    pub init_track: u32,
    /// The base health percentage at which the music changes to the boss track.
    pub bgm_change_percent: u32,
    /// The identifier of the music track played after the change, or negative when there is none.
    pub boss_track: i16,
    /// The reward scheme the stage uses on completion.
    pub rewards: RewardStructure,
    /// The difficulty rating shown in the stage selection interface.
    pub difficulty: u16,
    /// The highest crown difficulty the stage can be attempted at.
    pub max_crowns: u8,
    /// The crown difficulty the lineup restrictions apply at, or negative when they apply at every difficulty.
    pub target_crowns: i8,
    /// The enemy strength percentage applied at one crown, if the map declares it.
    pub crown_1_mag: Option<u32>,
    /// The enemy strength percentage applied at two crowns, if the map declares it.
    pub crown_2_mag: Option<u32>,
    /// The enemy strength percentage applied at three crowns, if the map declares it.
    pub crown_3_mag: Option<u32>,
    /// The enemy strength percentage applied at four crowns, if the map declares it.
    pub crown_4_mag: Option<u32>,
    /// A bitmask of the rarity tiers permitted, where each bit selects one tier.
    pub rarity_mask: u8,
    /// The greatest number of units deployable at once.
    pub deploy_limit: u32,
    /// A bitmask of the lineup rows permitted, where each bit selects one row.
    pub allowed_rows: u8,
    /// The lowest deployment cost a unit may have to be eligible.
    pub min_cost: u32,
    /// The highest deployment cost a unit may have to be eligible.
    pub max_cost: u32,
    /// The unit restriction group applied to this stage, if it declares one.
    pub charagroup: Option<CharaGroupEntry>,
    /// The predetermined lineups imposed at each crown difficulty, if any are.
    pub fixed_lineups: HashMap<u8, CertificationPreset>,
}

impl Stage {
    /// Returns the cost scheme the stage actually charges under.
    ///
    /// # Returns
    /// A `CostType` holding the scheme in effect, which is what the map declares
    /// unless the stage's chapter overrides it.
    pub fn cost_type(&self) -> CostType {
        CostType::of(&self.category, self.declared_cost_type)
    }

    /// Returns the stage's entry cost decoded into the currency it charges.
    ///
    /// # Returns
    /// A `ResolvedCost` holding the currency the stage charges and how much of it.
    pub fn resolved_cost(&self) -> ResolvedCost {
        resolve_energy(self.cost_type(), self.cost)
    }

    /// Returns the enemy strength percentage applied at a crown difficulty.
    ///
    /// The tiers run from one to four. A tier the map leaves undeclared, and any
    /// tier outside that range, reads as `None`.
    ///
    /// # Arguments
    /// * `crowns` - The crown difficulty the stage is being attempted at.
    ///
    /// # Returns
    /// An `Option` holding the percentage the tier declares, or `None` when it
    /// declares none.
    pub fn crown_magnification(&self, crowns: u8) -> Option<u32> {
        match crowns {
            1 => self.crown_1_mag,
            2 => self.crown_2_mag,
            3 => self.crown_3_mag,
            4 => self.crown_4_mag,
            _ => None,
        }
    }
}
