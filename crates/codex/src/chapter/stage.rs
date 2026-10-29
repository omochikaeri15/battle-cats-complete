mod background;
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

pub use background::{Background, BackgroundEntry, BackgroundError, Rgba};
pub use battleground::{Battleground, BattlegroundEntry, BattlegroundError, BossType, EnemyAmount};
pub use certification_preset::{AbilityType, CannonType, CertificationPreset, CertificationPresetError, EvolutionForm, PresetAbility, PresetChara, PresetTreasure, TreasureType};
pub use charagroup::{CharaGroup, CharaGroupEntry, CharaGroupError, CharaGroupType};
pub use cost::{resolve_energy, CataminGrade, CostType, ResolvedCost};
pub use difficulty_level::{DifficultyLevel, DifficultyLevelError};
pub use drop_chara::{DropChara, DropCharaError};
pub use fixed_formation::{FixedFormation, FixedFormationEntry, FixedFormationError};
pub use mapstagedata::{DropOdds, DropReward, MapStageData, MapStageDataEntry, MapStageDataError, MapStageDataHeader, RewardStructure, TimedScore};
pub use scatcpusetting::{ScatCpuSetting, ScatCpuSettingError};
pub use stage_option::{StageOption, StageOptionEntry, StageOptionError};
pub use stagename::{StageName, StageNameEntry, StageNameError};
pub use xp::get_hardcoded_xp;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stage {
    pub name: String,
    pub category: Category,
    pub map_id: u32,
    pub stage_id: u32,
    pub base_id: i32,
    pub anim_base_id: u32,
    pub width: u32,
    pub base_hp: u32,
    pub min_spawn: u32,
    pub max_spawn: u32,
    pub background_id: u32,
    pub max_enemies: u32,
    pub time_limit: u32,
    pub is_no_continues: bool,
    pub is_base_indestructible: bool,
    pub unknown_value: u32,
    pub enemies: Vec<BattlegroundEntry>,
    pub cost: u32,
    pub declared_cost_type: CostType,
    pub xp: u32,
    pub init_track: u32,
    pub bgm_change_percent: u32,
    pub boss_track: i16,
    pub rewards: RewardStructure,
    pub difficulty: u16,
    pub max_crowns: u8,
    pub target_crowns: i8,
    pub crown_1_mag: Option<u32>,
    pub crown_2_mag: Option<u32>,
    pub crown_3_mag: Option<u32>,
    pub crown_4_mag: Option<u32>,
    pub rarity_mask: u8,
    pub deploy_limit: u32,
    pub allowed_rows: u8,
    pub min_cost: u32,
    pub max_cost: u32,
    pub charagroup: Option<CharaGroupEntry>,
    pub fixed_lineups: HashMap<u8, CertificationPreset>,
}

impl Stage {
    pub fn cost_type(&self) -> CostType {
        CostType::of(&self.category, self.declared_cost_type)
    }

    pub fn resolved_cost(&self) -> ResolvedCost {
        resolve_energy(self.cost_type(), self.cost)
    }

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
