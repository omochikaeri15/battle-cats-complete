mod demoncastle;
mod dropitem;
mod ex_option;
mod invasion;
mod lockskipdata;
mod map_name;
mod map_option;
mod redirect;
mod scorebonusmap;
mod specialrulesmap;
mod specialrulesmapoption;

use serde::{Deserialize, Serialize};

use crate::chapter::Category;

pub use demoncastle::{DemonCastleDefine, DemonCastleError, DemonCastleLimit};
pub use dropitem::{DropItem, DropItemEntry, DropItemError};
pub use ex_option::{ExOption, ExOptionError};
pub use invasion::{fixed_invasion_target, InvasionTarget};
pub use lockskipdata::{LockSkipData, LockSkipDataEntry, LockSkipDataError};
pub use map_name::{MapName, MapNameError};
pub use map_option::{MapOption, MapOptionEntry, MapOptionError, ResetType};
pub use redirect::redirect_map_id;
pub use scorebonusmap::{BonusType, ScoreBonusMap, ScoreBonusMapEntry, ScoreBonusMapError};
pub use specialrulesmap::{RuleType, SpecialRulesMap, SpecialRulesMapEntry, SpecialRulesMapError};
pub use specialrulesmapoption::{SpecialRulesMapOption, SpecialRulesMapOptionEntry, SpecialRulesMapOptionError};

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Map {
    pub name: String,
    pub category: Category,
    pub map_id: u32,
    pub stages: Vec<u32>,
    pub max_crowns: u8,
    pub has_abyss: bool,
    pub crown_1_mag: Option<u32>,
    pub crown_2_mag: Option<u32>,
    pub crown_3_mag: Option<u32>,
    pub crown_4_mag: Option<u32>,
    pub reset_type: ResetType,
    pub max_clears: u32,
    pub cooldown_minutes: u32,
    pub hidden_upon_clear: bool,
    pub comment: String,
    pub ex_invasion: Option<u32>,
    pub score_bonuses: Option<ScoreBonusMapEntry>,
    pub special_rules: Option<SpecialRulesMapEntry>,
    pub invalid_combos: Vec<u32>,
    pub cost_multiplier: u32,
    pub drop_items: Option<DropItemEntry>,
}