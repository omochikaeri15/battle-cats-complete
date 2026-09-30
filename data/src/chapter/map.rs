//! Public facade for map-level data.
//!
//! This module re-exports the aggregate [`Map`] structure alongside every
//! specialized parser and error type that contributes to it.
mod demoncastle;
mod dropitem;
mod ex_option;
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
pub use lockskipdata::{LockSkipData, LockSkipDataEntry, LockSkipDataError};
pub use map_name::{MapName, MapNameError};
pub use map_option::{MapOption, MapOptionEntry, MapOptionError, ResetType};
pub use redirect::redirect_map_id;
pub use scorebonusmap::{BonusType, ScoreBonusMap, ScoreBonusMapEntry, ScoreBonusMapError};
pub use specialrulesmap::{RuleType, SpecialRulesMap, SpecialRulesMapEntry, SpecialRulesMapError};
pub use specialrulesmapoption::{SpecialRulesMapOption, SpecialRulesMapOptionEntry, SpecialRulesMapOptionError};

/// The fully-aggregated representation of a single map.
///
/// The engine scatters a map's definition across separate name, option, bonus,
/// rule, and drop tables. This structure combines them into one payload, leaving
/// as `None` the parts a given map does not participate in.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Map {
    /// The localized display name of the map.
    pub name: String,
    /// The chapter grouping this map belongs to.
    pub category: Category,
    /// The map's own identifier within its category.
    pub map_id: u32,
    /// The identifiers of the stages this map contains, in play order.
    pub stages: Vec<u32>,
    /// The highest crown difficulty the map can be attempted at.
    pub max_crowns: u8,
    /// Whether the map exposes the additional Abyss difficulty tier.
    pub has_abyss: bool,
    /// The enemy strength percentage applied at one crown, if declared.
    pub crown_1_mag: Option<u32>,
    /// The enemy strength percentage applied at two crowns, if declared.
    pub crown_2_mag: Option<u32>,
    /// The enemy strength percentage applied at three crowns, if declared.
    pub crown_3_mag: Option<u32>,
    /// The enemy strength percentage applied at four crowns, if declared.
    pub crown_4_mag: Option<u32>,
    /// What the map discards when its repeat timer elapses.
    pub reset_type: ResetType,
    /// The number of times the map may be cleared for rewards.
    pub max_clears: u32,
    /// The delay in minutes before the map becomes available again.
    pub cooldown_minutes: u32,
    /// Whether the map is removed from the selection list once cleared.
    pub hidden_upon_clear: bool,
    /// The trailing comment text accompanying the map's row in the source file.
    pub comment: String,
    /// The identifier of the EX map this map can divert into, if any.
    pub ex_invasion: Option<u32>,
    /// The score bonuses this map awards, if it declares any.
    pub score_bonuses: Option<ScoreBonusMapEntry>,
    /// The constraints this map imposes on the player, if it declares any.
    pub special_rules: Option<SpecialRulesMapEntry>,
    /// The combo effect types this map switches off.
    pub invalid_combos: Vec<u32>,
    /// The multiplier applied to every unit's base deployment cost, or zero when the map leaves costs alone.
    pub cost_multiplier: u32,
    /// The reward drop configuration for this map, if it declares one.
    pub drop_items: Option<DropItemEntry>,
}